#![feature(decl_macro)]
#![feature(impl_trait_in_assoc_type)]
#![feature(result_option_map_or_default)]

mod resources;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::proto::v1 as v1_svc;
use cos_api_shared::proto::v1 as v1_shared;
use cos_api_shared::{ReconcilableDriver, Resource, Specification, State};
use rtnetlink::new_connection;
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

use crate::resources::{LinkConfigSpec, NetworkResources};

struct NetworkManagerReconcilerService;

impl NetworkManagerReconcilerService {
    const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl v1_svc::ReconcilerService for NetworkManagerReconcilerService {
    async fn reconcile_resource(
        &self,
        request: Request<v1::ReconcileResourceRequest>,
    ) -> Result<Response<v1::ReconcileResourceResponse>, Status> {
        let mut res: NetworkResources = request
            .into_inner()
            .resource
            .try_into()
            .map_err(Status::internal)?;

        let (conn, mut handle, _) = new_connection().unwrap();
        tokio::spawn(conn);

        res.reconcile(handle).await;

        todo!()

        // Ok(Response::new(v1::ReconcileResourceResponse {
        //     state: res
        //         .state()
        //         .cloned()
        //         .map_or_default(|s| s.into_bytes().unwrap()),
        //     ..Default::default()
        // }))
    }

    async fn reconcile_delete(
        &self,
        request: Request<v1::ReconcileDeleteRequest>,
    ) -> Result<Response<v1::ReconcileDeleteResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse().unwrap();
    let svc = NetworkManagerReconcilerService::new();

    println!("NetworkReconcilerServer listening on {addr}");

    Server::builder()
        .add_service(v1_svc::ReconcilerServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
