use anyhow::Result;
use cos_proto_reconciler::v1::{
    ReconcileRequest,
    ReconcileResponse,
    ValidateRequest,
    ValidateResponse,
};
use cos_proto_reconciler::{Identity, Resource, SubResourceCreate};
use cos_proto_reconciler_server::v1::{
    ReconcilerService,
    ReconcilerServiceServer,
};
use serde_json::Value;
use system_controller::{StaticFileReconciler, StaticFileResource};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct Reconciler;

#[tonic::async_trait]
impl ReconcilerService for Reconciler {
    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        let req = request.into_inner();
        let (resource, maybe_resource): (
            SubResourceCreate<Value>,
            Option<Resource<Value, Value, Value>>,
        ) = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        let key = resource.id.key();

        match key.schema.as_ref() {
            "system:static-file" => {
                let spec = serde_json::from_value(resource.spec.clone())
                    .map_err(|err| Status::from_error(err.into()))?;

                let maybe_resource = maybe_resource
                    .map(|v| {
                        Ok::<_, anyhow::Error>(StaticFileResource {
                            id: v.id,
                            phase: v.phase,
                            status: v.status,
                            spec: serde_json::from_value(v.spec)?,
                            derived_spec: serde_json::from_value(
                                v.derived_spec,
                            )?,
                            state: v
                                .state
                                .map(serde_json::from_value)
                                .transpose()?,
                            children: v.children,
                            dependencies: v.dependencies,
                            dependents: v.dependents,
                        })
                    })
                    .transpose()
                    .map_err(|err| Status::from_error(err.into()))?;

                let reconciler = StaticFileReconciler::new_in("/etc".into());
                let response = reconciler
                    .validate(spec, maybe_resource)
                    .await
                    .map_err(|err| Status::from_error(err.into()))?;

                return Ok(Response::new(ValidateResponse {
                    raw: serde_json::to_vec(&response)
                        .map_err(|err| Status::from_error(err.into()))?,
                }));
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }

    async fn reconcile(
        &self,
        request: Request<ReconcileRequest>,
    ) -> Result<Response<ReconcileResponse>, Status> {
        let req = request.into_inner();
        let resource: Resource<Value, Value, Value> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        let key = resource.id.key();

        match key.schema.as_ref() {
            "system:static-file" => {
                let resource = StaticFileResource {
                    id: resource.id,
                    phase: resource.phase,
                    status: resource.status,
                    spec: serde_json::from_value(resource.spec)
                        .map_err(|err| Status::from_error(err.into()))?,
                    derived_spec: serde_json::from_value(resource.derived_spec)
                        .map_err(|err| Status::from_error(err.into()))?,
                    state: resource
                        .state
                        .map(serde_json::from_value)
                        .transpose()
                        .map_err(|err| Status::from_error(err.into()))?,
                    children: resource.children,
                    dependencies: resource.dependencies,
                    dependents: resource.dependents,
                };
                let reconciler = StaticFileReconciler::new_in("/etc".into());
                let response = reconciler
                    .reconcile(resource)
                    .await
                    .map_err(|err| Status::from_error(err.into()))?;

                return Ok(Response::new(ReconcileResponse {
                    raw: serde_json::to_vec(&response)
                        .map_err(|err| Status::from_error(err.into()))?,
                }));
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let addr = "[::1]:50051".parse()?;
    let reconciler = Reconciler;

    tracing::info!("system controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve(addr)
        .await?;

    Ok(())
}
