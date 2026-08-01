use anyhow::Result;
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
use system_controller::{StaticFileReconciler, StaticFileResource};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
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
                validate!(
                    resource,
                    maybe_resource,
                    StaticFileResource,
                    StaticFileReconciler::new_in("/etc".into())
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
            "system:static-file" => {
                reconcile!(
                    resource,
                    StaticFileResource,
                    StaticFileReconciler::new_in("/etc".into())
                );
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let addr = "[::1]:50051";
    let reconciler = Reconciler;

    let listener = TcpListener::bind(addr).await?;
    let incoming = TcpListenerStream::new(listener);
    tracing::info!("system controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
