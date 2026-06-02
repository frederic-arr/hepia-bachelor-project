/*
mod resources;

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use bollard::plugin::ContainerCreateBody;
use bollard::query_parameters::CreateContainerOptionsBuilder;
use bollard::{API_DEFAULT_VERSION, Docker};
use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    CreateDynamicResourceRequest,
    ReconcileDynamicResourceRequest,
};
use cos_api_reconciler_server::ReconcilableDriver;
use cos_api_reconciler_server::proto::v1 as v1_svc;
use cos_api_shared::proto::v1 as v1_shared;
use cos_api_shared::{Resource, Specification, State};
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

use crate::resources::{ContainerSpec, ContainerState};

struct NetworkManagerReconcilerService;

impl NetworkManagerReconcilerService {
    const fn new() -> Self {
        Self
    }
}

async fn create_connection() -> (Child, Docker) {
    let mut handle = Command::new("/usr/bin/podman")
        .args([
            "system",
            "service",
            "--time=0",
            "unix:///run/user/1000/podman2.sock",
        ])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;

    let connection = Docker::connect_with_unix(
        "/run/user/1000/podman2.sock",
        120,
        API_DEFAULT_VERSION,
    )
    .unwrap();

    (handle, connection)
}

#[async_trait]
impl v1_svc::ReconcilerService for NetworkManagerReconcilerService {
    async fn reconcile_user_config(
        &self,
        request: Request<v1::ReconcileUserConfigRequest>,
    ) -> Result<Response<v1::ReconcileUserConfigResponse>, Status> {
        todo!()
    }

    async fn create_dynamic_resource(
        &self,
        request: Request<v1::CreateDynamicResourceRequest>,
    ) -> Result<Response<v1::CreateDynamicResourceResponse>, Status> {
        let (handle, mut conn) = create_connection().await;
        let resource = request.into_inner();
        let schema = resource.id.clone().unwrap().schema;
        match schema.as_str() {
            ContainerSpec::SCHEMA => {
                let request =
                    CreateDynamicResourceRequest::<ContainerSpec>::from(
                        resource,
                    );

                let options = CreateContainerOptionsBuilder::default()
                    .name(request.id.name())
                    .build();

                let config = ContainerCreateBody {
                    image: Some("busybox".to_string()),
                    cmd: Some(vec!["/bin/sleep".to_string(), "1000".to_string()]),
                    ..Default::default()
                };
                let r =
                    conn.create_container(Some(options), config).await.unwrap();
                conn.start_container(request.id.name(), None).await.unwrap();

                Ok(Response::new(v1::CreateDynamicResourceResponse {
                    state: Some(
                        v1::create_dynamic_resource_response::State::Ready(
                            v1::create_dynamic_resource_response::StateReady {
                                state: rmp_serde::to_vec(
                                    &ContainerState::refresh(
                                        request.id,
                                        &request.spec,
                                        &mut conn,
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
        // TODO: Move this to some sort of global state?
        let (handle, mut conn) = create_connection().await;
        let resource = request.into_inner();
        let schema = resource.id.clone().unwrap().schema;
        match schema.as_str() {
            ContainerSpec::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<ContainerSpec>::from(
                        resource,
                    );

                let current = ContainerState::refresh(
                    request.id.clone(),
                    &request.spec,
                    &mut conn,
                )
                .await
                .unwrap();

                let mut needs_change = false;
                match (request.spec.running, current.running) {
                    (true, false) => {
                        conn.start_container(request.id.name(), None)
                            .await
                            .unwrap();

                        needs_change = true;
                    }
                    (false, true) => {
                        conn.stop_container(request.id.name(), None)
                            .await
                            .unwrap();

                        needs_change = true;
                    }
                    (true, true) | (false, false) => {
                        needs_change = false;
                    }
                }

                let state = if needs_change {
                    ContainerState::refresh(
                        request.id,
                        &request.spec,
                        &mut conn,
                    )
                    .await
                    .unwrap()
                } else {
                    current
                };

                Ok(Response::new(v1::ReconcileDynamicResourceResponse {
                    state: Some(v1::reconcile_dynamic_resource_response::State::Ready(v1::reconcile_dynamic_resource_response::StateReady{
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
    let addr = "[::1]:50053".parse().unwrap();
    let svc = NetworkManagerReconcilerService::new();

    println!("contianer-manager listening on {addr}");

    Server::builder()
        .add_service(v1_svc::ReconcilerServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
*/

fn main() {}
