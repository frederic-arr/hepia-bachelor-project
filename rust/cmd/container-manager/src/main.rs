#![feature(never_type)]

mod resources;

use std::collections::HashMap;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use bollard::query_parameters::{
    CreateContainerOptionsBuilder,
    ListImagesOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    Identity,
    ReconcileDynamicResourceRequest,
    ReconcileUserConfigRequest,
};
use cos_api_reconciler_server::Reconcilable;
use cos_api_reconciler_server::proto::v1 as v1_svc;
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

use crate::resources::{
    Container,
    ContainerConfig,
    ContainerConfigSpec,
    ContainerConfigState,
    ContainerSpec,
    ContainerState,
};

struct ContainerManagerReconcilerService {
    podman: Child,
    handle: Docker,
}

impl ContainerManagerReconcilerService {
    async fn new() -> Self {
        let (podman, handle) = create_connection().await;
        Self { podman, handle }
    }
}

async fn create_connection() -> (Child, Docker) {
    std::fs::create_dir_all("/etc/containers").unwrap();
    let mut f = std::fs::File::create("/etc/containers/policy.json").unwrap();
    f.write_all(
        br#"
    {
        "default": [
            {
                "type": "insecureAcceptAnything"
            }
        ]
    }
    "#,
    )
    .unwrap();

    let mut handle = Command::new("/bin/podman")
        .args(["system", "service", "--time=0", "tcp://127.0.0.1:9876"])
        .env("NETAVARK_FW", "nftables")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(5000)).await;

    let connection = Docker::connect_with_host("tcp://127.0.0.1:9876").unwrap();

    (handle, connection)
}

#[async_trait]
impl v1_svc::ReconcilerService for ContainerManagerReconcilerService {
    async fn reconcile_user_config(
        &self,
        request: Request<v1::ReconcileUserConfigRequest>,
    ) -> Result<Response<v1::ReconcileUserConfigResponse>, Status> {
        let request = request.into_inner();
        match request.schema.as_str() {
            ContainerConfig::SCHEMA => {
                let request = ReconcileUserConfigRequest::<
                    ContainerConfigSpec,
                    ContainerConfigState,
                > {
                    schema: request.schema,
                    name: request.name,
                    spec: rmp_serde::from_slice(&request.spec).unwrap(),
                    state: request.state.and_then(|s| match s {
                        v1::reconcile_user_config_request::State::Unset(()) => {
                            None
                        }
                        v1::reconcile_user_config_request::State::Ready(s) => {
                            rmp_serde::from_slice(&request.spec).unwrap()
                        }
                    }),
                    children: request
                        .children
                        .into_iter()
                        .map(|c| Identity {
                            schema: c.schema,
                            name: c.name,
                        })
                        .collect(),
                };

                let response = ContainerConfig::reconcile(&mut (), &request)
                    .await
                    .expect("not possible to have errors");
                Ok(Response::new(response))
            }
            _ => todo!(),
        }
    }

    async fn reconcile_dynamic_resource(
        &self,
        request: Request<v1::ReconcileDynamicResourceRequest>,
    ) -> Result<Response<v1::ReconcileDynamicResourceResponse>, Status> {
        let request = request.into_inner();
        match request.schema.as_str() {
            Container::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<ContainerSpec, ContainerState> {
                        schema: request.schema,
                        name: request.name,
                        spec: rmp_serde::from_slice(&request.spec).unwrap(),
                        state: request.state.and_then(|s| match s {
                            v1::reconcile_dynamic_resource_request::State::Unset(
                                (),
                            ) => None,
                            v1::reconcile_dynamic_resource_request::State::Ready(
                                s,
                            ) => rmp_serde::from_slice(&request.spec).unwrap(),
                        }),
                        owner: request.owner.map(|o| Identity {
                            schema: o.schema,
                            name: o.name,
                        }).unwrap(),
                        children: request
                            .children
                            .into_iter()
                            .map(|c| Identity {
                                schema: c.schema,
                                name: c.name,
                            })
                            .collect(),
                    };

                let response =
                    Container::reconcile(&mut self.handle.clone(), &request)
                        .await
                        .map_err(Status::internal)?;
                Ok(Response::new(response))
            }
            _ => todo!(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50053".parse().unwrap();
    let svc = ContainerManagerReconcilerService::new().await;

    println!("contianer-manager listening on {addr}");

    Server::builder()
        .add_service(v1_svc::ReconcilerServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
