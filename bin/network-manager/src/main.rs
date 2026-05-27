mod resources;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    CreateDynamicResourceRequest,
    ReconcileDynamicResourceRequest,
};
use cos_api_reconciler_server::ReconcilableDriver;
use cos_api_reconciler_server::proto::v1 as v1_svc;
use cos_api_shared::proto::v1 as v1_shared;
use cos_api_shared::{Resource, Specification, State};
use rtnetlink::{LinkDummy, new_connection};
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

use crate::resources::{LinkSpec, LinkState};

struct NetworkManagerReconcilerService;

impl NetworkManagerReconcilerService {
    const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl v1_svc::ReconcilerService for NetworkManagerReconcilerService {
    async fn create_dynamic_resource(
        &self,
        request: Request<v1::CreateDynamicResourceRequest>,
    ) -> Result<Response<v1::CreateDynamicResourceResponse>, Status> {
        let (conn, mut rtnl, _) = new_connection().unwrap();
        tokio::spawn(conn);

        let resource = request.into_inner();
        let schema = resource.id.clone().unwrap().schema;
        match schema.as_str() {
            LinkSpec::SCHEMA => {
                let request =
                    CreateDynamicResourceRequest::<LinkSpec>::from(resource);

                let mut msg = LinkDummy::new(request.id.name());
                if request.spec.admin_up {
                    msg = msg.up();
                } else {
                    msg = msg.down();
                }

                rtnl.link().add(msg.build()).execute().await.unwrap();
                Ok(Response::new(v1::CreateDynamicResourceResponse {
                    state: Some(
                        v1::create_dynamic_resource_response::State::Ready(
                            v1::StateReady {
                                state: rmp_serde::to_vec(
                                    &LinkState::refresh(
                                        request.id,
                                        &request.spec,
                                        &mut rtnl,
                                    )
                                    .await
                                    .unwrap(),
                                )
                                .unwrap(),
                            },
                        ),
                    ),
                }))
            }
            _ => todo!(),
        }
    }

    async fn reconcile_dynamic_resource(
        &self,
        request: Request<v1::ReconcileDynamicResourceRequest>,
    ) -> Result<Response<v1::ReconcileDynamicResourceResponse>, Status> {
        let (conn, mut rtnl, _) = new_connection().unwrap();
        tokio::spawn(conn);

        let resource = request.into_inner();
        let schema = resource.id.clone().unwrap().schema;
        match schema.as_str() {
            LinkSpec::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<LinkSpec>::from(resource);

                let current = LinkState::refresh(
                    request.id.clone(),
                    &request.spec,
                    &mut rtnl,
                )
                .await
                .unwrap();

                let mut msg = LinkDummy::new(request.id.name());
                let needs_change =
                    match (request.spec.admin_up, current.admin_up) {
                        (true, false) => {
                            msg = msg.up();
                            true
                        }
                        (false, true) => {
                            msg = msg.down();
                            true
                        }
                        (true, true) | (false, false) => false,
                    };

                if needs_change {
                    rtnl.link().change(msg.build()).execute().await.unwrap();
                }

                let state = if needs_change {
                    LinkState::refresh(request.id, &request.spec, &mut rtnl)
                        .await
                        .unwrap()
                } else {
                    current
                };

                Ok(Response::new(v1::ReconcileDynamicResourceResponse {
                    state: Some(v1::reconcile_dynamic_resource_response::State::Ready(v1::StateReady {
                        state: rmp_serde::to_vec(
                            &state
                        )
                        .unwrap(),
                    })),
                }))
            }
            _ => todo!(),
        }
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
