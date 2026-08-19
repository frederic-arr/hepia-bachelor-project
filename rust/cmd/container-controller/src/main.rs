use anyhow::Result;
use container_controller::{
    ImageReconciler,
    ImageResource,
    InstanceReconciler,
    InstanceResource,
    NetworkReconciler,
    NetworkResource,
    RuntimeReconciler,
    RuntimeResource,
    VolumeReconciler,
    VolumeResource,
};
use cos_proto_reconciler::v1::{
    ReconcileRequest,
    ReconcileResponse,
    ValidateRequest,
    ValidateResponse,
};
use cos_proto_reconciler::{Resource, SubResourceCreate};
use cos_proto_reconciler_server::v1::{
    ReconcilerService,
    ReconcilerServiceServer,
};
use cos_proto_reconciler_server::{reconcile, validate};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct Reconciler;

// ! container-controller must be able to grant all capabilities to children
// container

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
            "container:runtime" => {
                validate!(
                    resource,
                    maybe_resource,
                    RuntimeResource,
                    RuntimeReconciler::new()
                );
            }
            "container:instance" => {
                validate!(
                    resource,
                    maybe_resource,
                    InstanceResource,
                    InstanceReconciler::new()
                );
            }
            "container:image" => {
                validate!(
                    resource,
                    maybe_resource,
                    ImageResource,
                    ImageReconciler::new()
                );
            }
            "container:network" => {
                validate!(
                    resource,
                    maybe_resource,
                    NetworkResource,
                    NetworkReconciler::new()
                );
            }
            "container:volume" => {
                validate!(
                    resource,
                    maybe_resource,
                    VolumeResource,
                    VolumeReconciler::new()
                );
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
            "container:runtime" => {
                reconcile!(
                    resource,
                    RuntimeResource,
                    RuntimeReconciler::new()
                );
            }
            "container:instance" => {
                reconcile!(
                    resource,
                    InstanceResource,
                    InstanceReconciler::new()
                );
            }
            "container:image" => {
                reconcile!(resource, ImageResource, ImageReconciler::new());
            }
            "container:network" => {
                reconcile!(
                    resource,
                    NetworkResource,
                    NetworkReconciler::new()
                );
            }
            "container:volume" => {
                reconcile!(resource, VolumeResource, VolumeReconciler::new());
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let addr = "127.0.0.1:50053";
    let reconciler = Reconciler;

    let listener = TcpListener::bind(addr).await?;
    let incoming = TcpListenerStream::new(listener);
    tracing::info!("container controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
