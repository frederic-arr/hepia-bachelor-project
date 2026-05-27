use cos_api_shared::{Reconcilable, Resource, Specification, State};
use derive_builder::Builder;
use futures::StreamExt;
use rtnetlink::packet_route::link;
use rtnetlink::packet_route::link::{LinkAttribute, LinkMessage};
use rtnetlink::{Handle, LinkDummy};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkConfigSpec {
    pub state: LinkConfigSpecState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkConfigSpecState {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct LinkConfigState {
    pub index: u32,
    pub oper_state: LinkConfigStateOperState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkConfigStateOperState {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkPlan {
    Create(LinkMessage),
    Modify(LinkMessage),
    Delete(u32),
    Nop,
}

impl Specification for LinkConfigSpec {
    type State = LinkConfigState;

    const SCHEMA: &str = ".containeros.net.link-config";
}

impl State for LinkConfigState {}

impl Reconcilable for LinkConfigSpec {
    type CurrentState = <Self as Specification>::State;
    type Data = Handle;
    type Error = String;
    type Output = ();
    type Plan = LinkPlan;

    fn refresh(
        resource: &Resource<Self>,
        data: &mut Self::Data,
    ) -> impl Future<Output = Result<Option<Self::CurrentState>, Self::Error>> + Send
    {
        async {
            let mut state = LinkConfigStateBuilder::default();

            let mut links = data
                .link()
                .get()
                .match_name(resource.id().name().clone())
                .execute();

            let link = links.next().await.expect("at least one RTNL message");
            let link = match link {
                Ok(v) => v,
                Err(rtnetlink::Error::NetlinkError(err)) => {
                    if let Some(code) = err.code
                        && Errno::from_raw_os_error(code.abs().into())
                            == Errno::NODEV
                    {
                        return Ok(None);
                    }

                    return Err(format!("{err}"));
                }
                Err(err) => return Err(format!("{err}")),
            };

            state.index(link.header.index);
            for nla in link.attributes {
                match nla {
                    LinkAttribute::OperState(link::State::Up) => {
                        state.oper_state(LinkConfigStateOperState::Up);
                    }
                    LinkAttribute::OperState(link::State::Down) => {
                        state.oper_state(LinkConfigStateOperState::Down);
                    }
                    LinkAttribute::OperState(_) => {
                        state.oper_state(LinkConfigStateOperState::Unknown);
                    }
                    _ => {}
                }
            }

            state.build().map(Some).map_err(|err| format!("{err}"))
        }
    }

    fn plan(
        resource: &Resource<Self>,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error> {
        let mut msg = LinkDummy::new(&resource.id().name());
        let empty = LinkDummy::new(&resource.id().name()).build();

        let desired_oper = &resource.spec().state;
        let current_oper = state
            .map_or(LinkConfigStateOperState::Unknown, |c| {
                c.oper_state.clone()
            });

        match (desired_oper, current_oper) {
            (LinkConfigSpecState::Up, LinkConfigStateOperState::Down)
            | (LinkConfigSpecState::Up, LinkConfigStateOperState::Unknown) => {
                msg = msg.up();
            }

            (LinkConfigSpecState::Down, LinkConfigStateOperState::Up)
            | (LinkConfigSpecState::Down, LinkConfigStateOperState::Unknown) => {
                msg = msg.down();
            }

            (LinkConfigSpecState::Down, LinkConfigStateOperState::Down)
            | (LinkConfigSpecState::Up, LinkConfigStateOperState::Up) => {}
        }

        let msg = msg.build();
        if msg == empty {
            return Ok(LinkPlan::Nop);
        }

        match state {
            Some(_) => Ok(LinkPlan::Modify(msg)),
            None => Ok(LinkPlan::Create(msg)),
        }
    }

    fn apply(
        resource: &Resource<Self>,
        data: &mut Self::Data,
        plan: Self::Plan,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> {
        async {
            match plan {
                LinkPlan::Create(msg) => data
                    .link()
                    .add(msg)
                    .execute()
                    .await
                    .map_err(|err| format!("{err}")),

                LinkPlan::Modify(msg) => data
                    .link()
                    .change(msg)
                    .execute()
                    .await
                    .map_err(|err| format!("{err}")),

                LinkPlan::Delete(index) => data
                    .link()
                    .del(index)
                    .execute()
                    .await
                    .map_err(|err| format!("{err}")),

                LinkPlan::Nop => return Ok(()),
            }
        }
    }
}
