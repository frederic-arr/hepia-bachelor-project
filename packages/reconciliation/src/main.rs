use std::sync::Arc;
use std::time::{Duration, Instant};

use rtnetlink::sys::SmolSocket;
use rtnetlink::{Handle, new_connection_with_socket};
use smol::Timer;
use smol_macros::main;

use crate::link::spec;
use crate::reconcilable::{Phase, Reconcilable};

main! {
    async fn main() {
        reconciliation_loop().await;
    }
}

async fn reconciliation_loop() {
    let (conn, handle, _) = new_connection_with_socket::<SmolSocket>().unwrap();
    let task = smol::spawn(conn);
    task.detach();

    let rt = Arc::new(handle);
    loop {
        sync(rt.clone()).await;
        Timer::after(Duration::from_secs(5)).await;
    }
}

async fn sync(rt: Arc<Handle>) {
    let start = Instant::now();

    // 1. Get desired state
    println!("[DESIRED]");
    let mut link = link::LinkConfig {
        spec: Phase::Running(spec::Link {
            name: "dummy0".to_string(),
            state: spec::LinkState::Down,
        }),
        rtnl: rt,
    };
    dbg!(&link);

    // 2. Get current state
    println!("[REFRESH]");
    let state = link.refresh().await;
    dbg!(&state);

    // 3. Compare
    println!("[COMPARE]");
    let plan = link.plan(&state).await;
    dbg!(&plan);

    // 4. Act
    let state = link.apply(state, plan).await;
    dbg!(&state);

    let elapsed = start.elapsed();
    dbg!(elapsed);
}

mod reconcilable {
    pub trait Reconcilable {
        type State;
        type Plan;

        async fn refresh(&mut self) -> Self::State;
        async fn plan(&mut self, state: &Self::State) -> Self::Plan;
        async fn apply(
            &mut self,
            state: Self::State,
            plan: Self::Plan,
        ) -> Self::State;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Phase<T> {
        Running(T),
        Teardown(T),
    }

    impl<T> Phase<T> {
        pub fn into_inner(&self) -> &T {
            match self {
                Phase::Running(v) => v,
                Phase::Teardown(v) => v,
            }
        }
    }
}

mod link {
    use std::sync::Arc;

    use futures_lite::StreamExt;
    use rtnetlink::packet_route::link::{LinkAttribute, LinkMessage, State};
    use rtnetlink::{Handle, LinkDummy};
    use rustix::io::Errno;

    use super::reconcilable::*;

    pub mod spec {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Link {
            pub name: String,
            pub state: LinkState,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum LinkState {
            Up,
            Down,
        }
    }

    pub mod rt {
        use derive_builder::Builder;

        #[derive(Debug, Clone, PartialEq, Eq, Builder)]
        #[builder(pattern = "mutable")]
        pub struct Link {
            pub index: u32,

            pub name: String,

            #[builder(default)]
            pub state: LinkState,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum LinkState {
            Up,
            Down,

            #[default]
            Unknown,
        }
    }

    #[derive(Debug, Clone)]
    pub struct LinkConfig {
        pub spec: Phase<spec::Link>,
        pub rtnl: Arc<Handle>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LinkPlan {
        Create(LinkMessage),
        Modify(LinkMessage),
        Delete(u32),
        Nop,
    }

    impl Reconcilable for LinkConfig {
        type Plan = LinkPlan;
        type State = Option<rt::Link>;

        async fn refresh(&mut self) -> Self::State {
            let spec = self.spec.into_inner();
            let mut state = rt::LinkBuilder::default();

            let mut links = self
                .rtnl
                .link()
                .get()
                .match_name(spec.name.clone())
                .execute();

            let link = links.next().await.expect("at least one RTNL message");
            let link = match link {
                Ok(v) => v,
                Err(rtnetlink::Error::NetlinkError(err)) => {
                    if let Some(code) = err.code
                        && Errno::from_raw_os_error(code.abs().into())
                            == Errno::NODEV
                    {
                        return None;
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
                        state.state(rt::LinkState::Up);
                    }
                    LinkAttribute::OperState(State::Down) => {
                        state.state(rt::LinkState::Down);
                    }
                    LinkAttribute::OperState(_) => {
                        state.state(rt::LinkState::Unknown);
                    }
                    _ => {}
                }
            }

            Some(state.build().unwrap())
        }

        async fn plan(&mut self, state: &Self::State) -> Self::Plan {
            match (&self.spec, state) {
                (Phase::Teardown(_), None) => LinkPlan::Nop,
                (Phase::Teardown(_), Some(cur)) => LinkPlan::Delete(cur.index),
                (Phase::Running(spec), cur) => {
                    let mut msg = LinkDummy::new(&spec.name);
                    let empty = LinkDummy::new(&spec.name).build();

                    if spec.state == spec::LinkState::Up
                        && cur
                            .as_ref()
                            .map_or(rt::LinkState::Unknown, |c| c.state)
                            != rt::LinkState::Up
                    {
                        msg = msg.up();
                    } else if spec.state == spec::LinkState::Down
                        && cur
                            .as_ref()
                            .map_or(rt::LinkState::Unknown, |c| c.state)
                            != rt::LinkState::Down
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
                }
            }
        }

        async fn apply(
            &mut self,
            state: Self::State,
            plan: Self::Plan,
        ) -> Self::State {
            match plan {
                LinkPlan::Create(msg) => {
                    self.rtnl.link().add(msg).execute().await.unwrap()
                }
                LinkPlan::Modify(msg) => {
                    self.rtnl.link().change(msg).execute().await.unwrap()
                }
                LinkPlan::Delete(index) => {
                    self.rtnl.link().del(index).execute().await.unwrap()
                }
                LinkPlan::Nop => return state,
            };

            self.refresh().await
        }
    }
}
