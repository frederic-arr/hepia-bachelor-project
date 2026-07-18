use anyhow::Result;
use cos_proto_reconciler::v1::{
    ReconcileRequest,
    ReconcileResponse,
    ValidateRequest,
    ValidateResponse,
};
use cos_proto_reconciler::{Identity, Resource};
use cos_proto_reconciler_server::v1::{
    ReconcilerService,
    ReconcilerServiceServer,
};
use network_controller::{DnsReconciler, DnsResource};
use serde_json::Value;
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
        let resource: Resource<Value, Option<Value>, Value> =
            serde_json::from_slice(&req.raw).unwrap();

        let key = match &resource.id {
            Identity::Static(key) => key,
            Identity::Dynamic(key) => key,
            Identity::Shared(key) => key,
        };

        match key.schema.as_ref() {
            "network:dns" => {
                let spec = serde_json::from_value(resource.spec.clone())
                    .map_err(|err| Status::from_error(err.into()))?;

                let maybe_resource = match resource.derived_spec {
                    Some(derived_spec) => {
                        let resource = DnsResource {
                            id: resource.id,
                            phase: resource.phase,
                            status: resource.status,
                            spec: serde_json::from_value(resource.spec)
                                .map_err(|err| {
                                    Status::from_error(err.into())
                                })?,
                            derived_spec: serde_json::from_value(derived_spec)
                                .map_err(|err| {
                                    Status::from_error(err.into())
                                })?,
                            state: resource
                                .state
                                .map(serde_json::from_value)
                                .transpose()
                                .map_err(|err| {
                                    Status::from_error(err.into())
                                })?,
                            children: resource.children,
                            dependencies: resource.dependencies,
                            dependents: resource.dependents,
                        };

                        Some(resource)
                    }
                    _ => None,
                };

                let mut reconciler = DnsReconciler::new();
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

        let v: Value = serde_json::from_slice(&req.raw).unwrap();
        println!("{v:#}");

        let resource: Resource<Value, Value, Value> =
            serde_json::from_slice(&req.raw).unwrap();

        let key = match &resource.id {
            Identity::Static(key) => key,
            Identity::Dynamic(key) => key,
            Identity::Shared(key) => key,
        };

        match key.schema.as_ref() {
            "network:dns" => {
                let resource = DnsResource {
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
                let mut reconciler = DnsReconciler::new();
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

    let addr = "[::1]:50052".parse().unwrap();
    let reconciler = Reconciler::default();

    tracing::info!("network controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve(addr)
        .await?;

    Ok(())
}
