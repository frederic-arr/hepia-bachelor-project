// mod model;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::proto::v1::{self as v1_svc, ReconcilerService};
use cos_api_shared::proto::v1::{DynamicResource, MetaResource};
use cos_api_shared::*;
use rtnetlink::new_connection;
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};

enum Specs {
    LinkConfig,
}

struct NetworkManagerReconcilerService;

impl NetworkManagerReconcilerService {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl v1_svc::ReconcilerService for NetworkManagerReconcilerService {
    async fn reconcile_resource(
        &self,
        request: Request<v1::ReconcileResourceRequest>,
    ) -> Result<Response<v1::ReconcileResourceResponse>, Status> {
        dbg!(&request);
        let mut res = request
            .into_inner()
            .resource
            .try_into()
            .map_err(|e| Status::internal(e))?;

        dbg!(&res);

        let (conn, handle, _) = new_connection().unwrap();
        let task = tokio::spawn(conn);

        link::refresh(handle.clone(), &mut res).await;
        dbg!(&res);

        let plan = link::plan(&mut res).await;
        dbg!(&plan);

        link::apply(handle, &mut res, plan).await;
        dbg!(res);

        Ok(Response::new(v1::ReconcileResourceResponse {
            ..Default::default()
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50052".parse().unwrap();
    let svc = NetworkManagerReconcilerService::new();

    let mut spec = link::spec::Link {
        name: "dummy0".to_string(),
        state: link::spec::LinkState::Down,
    };

    svc.reconcile_resource(Request::new(v1::ReconcileResourceRequest {
        resource: Some(MetaResource {
            resource_type: Some(
                proto::v1::meta_resource::ResourceType::Dynamic(
                    DynamicResource {
                        meta: Some(proto::v1::ResourceMeta {
                            id: Some(Default::default()),
                            children: vec![],
                            spec: rmp_serde::to_vec(&spec).unwrap(),
                            status: vec![],
                        }),
                        owner: Some(Default::default()),
                        dependencies: vec![],
                        dependents: vec![],
                    },
                ),
            ),
        }),
        additional_resources: vec![],
    }))
    .await
    .unwrap();

    // println!("NetworkReconcilerServer listening on {addr}");

    // Server::builder()
    //     .add_service(v1_svc::ReconcilerServiceServer::new(svc))
    //     .serve(addr)
    //     .await?;

    Ok(())
}

// use std::sync::Arc;
// use std::time::{Duration, Instant};

// use cos_api_shared::Identity;
// use cos_api_sysmgr::proto::v1::ResourceReadRequest;
// use cos_api_sysmgr_client::proto::v1::SystemManagerServiceClient;
// use rtnetlink::sys::SmolSocket;
// use rtnetlink::{Handle, new_connection_with_socket};
// use tonic::transport::Channel;

// use crate::link::spec;
// use crate::reconcilable::{Phase, Reconcilable};

// #[derive(Debug, Clone)]
// struct Rt {
//     handle: Handle,
//     client: SystemManagerServiceClient<Channel>,
// }

// #[tokio::main]
// async fn main() {
//     let (conn, handle, _) =
// new_connection_with_socket::<SmolSocket>().unwrap();     let client = SystemManagerServiceClient::connect("http://[::1]:50051")
//         .await
//         .unwrap();

//     let mut rt = Rt { handle, client };

//     loop {
//         sync(&mut rt).await;
//         tokio::time::sleep(Duration::from_secs(5)).await;
//     }
// }

// async fn sync(rt: &mut Rt) {
//     let start = Instant::now();

//     // 1. Get desired state
//     println!("[DESIRED]");
//     let res = rt
//         .client
//         .resource_read(ResourceReadRequest {
//             id: Some(
//                 Identity::new("netmgr".to_string(), "LinkConfig".to_string())
//                     .into(),
//             ),
//         })
//         .await;
//     dbg!(res);

//     // let mut link = link::LinkConfig {
//     //     spec: Phase::Running(spec::Link {
//     //         name: "dummy0".to_string(),
//     //         state: spec::LinkState::Down,
//     //     }),
//     //     rt,
//     // };
//     // dbg!(&link);

//     // // 2. Get current state
//     // println!("[REFRESH]");
//     // let state = link.refresh().await;
//     // dbg!(&state);

//     // // 3. Compare
//     // println!("[COMPARE]");
//     // let plan = link.plan(&state).await;
//     // dbg!(&plan);

//     // // 4. Act
//     // let state = link.apply(state, plan).await;
//     // dbg!(&state);

//     // let elapsed = start.elapsed();
//     // dbg!(elapsed);
// }

// mod reconcilable {
//     pub trait Reconcilable {
//         type State;
//         type Plan;

//         async fn refresh(&mut self) -> Self::State;
//         async fn plan(&mut self, state: &Self::State) -> Self::Plan;
//         async fn apply(
//             &mut self,
//             state: Self::State,
//             plan: Self::Plan,
//         ) -> Self::State;
//     }

//     #[derive(Debug, Clone, PartialEq, Eq)]
//     pub enum Phase<T> {
//         Running(T),
//         Teardown(T),
//     }

//     impl<T> Phase<T> {
//         pub fn into_inner(&self) -> &T {
//             match self {
//                 Phase::Running(v) => v,
//                 Phase::Teardown(v) => v,
//             }
//         }
//     }
// }

mod link {
    use std::sync::Arc;

    use cos_api_shared::Resource;
    use futures::StreamExt;
    use rtnetlink::packet_route::link::{LinkAttribute, LinkMessage, State};
    use rtnetlink::{Handle, LinkDummy};
    use rustix::io::Errno;

    // use super::reconcilable::*;

    pub mod spec {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
        pub struct Link {
            pub name: String,
            pub state: LinkState,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
        pub enum LinkState {
            Up,
            Down,
        }
    }

    pub mod status {
        use derive_builder::Builder;
        use serde::{Deserialize, Serialize};

        #[derive(
            Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize,
        )]
        #[builder(pattern = "mutable")]
        pub struct Link {
            pub index: u32,

            pub name: String,

            #[builder(default)]
            pub state: LinkState,
        }

        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize,
        )]
        pub enum LinkState {
            Up,
            Down,

            #[default]
            Unknown,
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LinkPlan {
        Create(LinkMessage),
        Modify(LinkMessage),
        Delete(u32),
        Nop,
    }

    pub type Res = Resource<spec::Link, status::Link>;
    pub async fn refresh(rtnl: Handle, res: &mut Res) {
        let spec = res.spec();
        let mut state = status::LinkBuilder::default();

        let mut links =
            rtnl.link().get().match_name(spec.name.clone()).execute();

        let link = links.next().await.expect("at least one RTNL message");
        let link = match link {
            Ok(v) => v,
            Err(rtnetlink::Error::NetlinkError(err)) => {
                if let Some(code) = err.code
                    && Errno::from_raw_os_error(code.abs().into())
                        == Errno::NODEV
                {
                    return;
                }

                panic!("Got err: {err}");
            }
            Err(err) => panic!("Got err: {err}"),
        };

        state.index(link.header.index);
        for nla in link.attributes {
            match nla {
                LinkAttribute::IfName(name) => {
                    state.name(name);
                }
                LinkAttribute::OperState(State::Up) => {
                    state.state(status::LinkState::Up);
                }
                LinkAttribute::OperState(State::Down) => {
                    state.state(status::LinkState::Down);
                }
                LinkAttribute::OperState(_) => {
                    state.state(status::LinkState::Unknown);
                }
                _ => {}
            }
        }

        let state = state.build().unwrap();
        res.status_opt_mut().replace(state);
    }

    pub async fn plan(res: &mut Res) -> LinkPlan {
        let mut msg = LinkDummy::new(&res.spec().name);
        let empty = LinkDummy::new(&res.spec().name).build();
        let cur = res.status();

        if res.spec().state == spec::LinkState::Up
            && cur.as_ref().map_or(status::LinkState::Unknown, |c| c.state)
                != status::LinkState::Up
        {
            msg = msg.up();
        } else if res.spec().state == spec::LinkState::Down
            && cur.as_ref().map_or(status::LinkState::Unknown, |c| c.state)
                != status::LinkState::Down
        {
            msg = msg.down();
        }

        let msg = msg.build();
        if msg == empty {
            return LinkPlan::Nop;
        }

        match cur {
            Some(_) => LinkPlan::Modify(msg),
            None => LinkPlan::Create(msg),
        }

        // match (spec, res.status()) {
        //     (Phase::Teardown(_), None) => LinkPlan::Nop,
        //     (Phase::Teardown(_), Some(cur)) => LinkPlan::Delete(cur.index),
        //     (Phase::Running(spec), cur) => {
        //         let mut msg = LinkDummy::new(&spec.name);
        //         let empty = LinkDummy::new(&spec.name).build();

        //         if spec.state == spec::LinkState::Up
        //             && cur
        //                 .as_ref()
        //                 .map_or(status::LinkState::Unknown, |c| c.state)
        //                 != status::LinkState::Up
        //         {
        //             msg = msg.up();
        //         } else if spec.state == spec::LinkState::Down
        //             && cur
        //                 .as_ref()
        //                 .map_or(status::LinkState::Unknown, |c| c.state)
        //                 != status::LinkState::Down
        //         {
        //             msg = msg.down();
        //         }

        //         let msg = msg.build();
        //         if msg == empty {
        //             return LinkPlan::Nop;
        //         }

        //         match cur {
        //             Some(_) => LinkPlan::Modify(msg),
        //             None => LinkPlan::Create(msg),
        //         }
        //     }
        // }
    }

    pub async fn apply(rtnl: Handle, res: &mut Res, plan: LinkPlan) {
        match plan {
            LinkPlan::Create(msg) => {
                rtnl.link().add(msg).execute().await.unwrap()
            }
            LinkPlan::Modify(msg) => {
                rtnl.link().change(msg).execute().await.unwrap()
            }
            LinkPlan::Delete(index) => {
                rtnl.link().del(index).execute().await.unwrap()
            }
            LinkPlan::Nop => return,
        };

        refresh(rtnl, res).await
    }
}
