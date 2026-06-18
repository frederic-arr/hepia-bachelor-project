#![feature(never_type)]

mod resources;

use std::io::Write;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    Identity,
    ReconcileDynamicResourceRequest,
    ReconcileUserConfigRequest,
};
use cos_api_reconciler_server::Reconcilable;
use cos_api_reconciler_server::proto::v1 as v1_svc;
use rtnetlink::{Handle, LinkMessageBuilder, LinkUnspec, new_connection};
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

use crate::resources::{
    Address,
    AddressSpec,
    AddressState,
    Link,
    LinkConfig,
    LinkConfigSpec,
    LinkConfigState,
    LinkSpec,
    LinkState,
    Route,
    RouteSpec,
    RouteState,
};

struct NetworkManagerReconcilerService {
    rtnl: Handle,
}

impl NetworkManagerReconcilerService {
    fn new() -> Self {
        let mut f = std::fs::File::create("/etc/resolv.conf").unwrap();
        f.write_all(
            br"
    nameserver 9.9.9.9
    ",
        )
        .unwrap();

        let (conn, mut rtnl, _) = new_connection().unwrap();
        tokio::spawn(conn);

        Self { rtnl }
    }
}

#[async_trait]
impl v1_svc::ReconcilerService for NetworkManagerReconcilerService {
    async fn reconcile_user_config(
        &self,
        request: Request<v1::ReconcileUserConfigRequest>,
    ) -> Result<Response<v1::ReconcileUserConfigResponse>, Status> {
        let request = request.into_inner();
        match request.schema.as_str() {
            LinkConfig::SCHEMA => {
                let request = ReconcileUserConfigRequest::<
                    LinkConfigSpec,
                    LinkConfigState,
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

                let response = LinkConfig::reconcile(&mut (), &request)
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
            Link::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<LinkSpec, LinkState> {
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
                    Link::reconcile(&mut self.rtnl.clone(), &request)
                        .await
                        .map_err(|v| Status::internal(v))?;
                Ok(Response::new(response))
            }
            Address::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<AddressSpec, AddressState> {
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
                    Address::reconcile(&mut self.rtnl.clone(), &request)
                        .await
                        .map_err(|v| Status::internal(v))?;
                Ok(Response::new(response))
            }
            Route::SCHEMA => {
                let request =
                    ReconcileDynamicResourceRequest::<RouteSpec, RouteState> {
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
                    Route::reconcile(&mut self.rtnl.clone(), &request)
                        .await
                        .map_err(|v| Status::internal(v))?;
                Ok(Response::new(response))
            }
            _ => todo!(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50052".parse().unwrap();
    let svc = NetworkManagerReconcilerService::new();

    println!("network-manager listening on {addr}");

    Server::builder()
        .add_service(v1_svc::ReconcilerServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
