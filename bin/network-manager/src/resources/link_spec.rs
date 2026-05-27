use cos_api_reconciler::proto::v1::ReconcileResourceResponse;
use cos_api_reconciler_server::{Reconcilable, ReconcilableDriver};
use cos_api_shared::{
    DynamicResource,
    Resource,
    ResourceSpec,
    Specification,
    State, UserConfigResource,
};
use derive_builder::Builder;
use futures::StreamExt;
use rtnetlink::packet_route::link::{
    self,
    LinkAttribute,
    LinkFlags,
    LinkMessage,
};
use rtnetlink::{Handle, LinkDummy};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkSpec {
    pub admin_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct LinkSpecState {
    pub index: u32,
    pub running: bool,
    pub admin_up: bool,
    pub oper_state: LinkSpecStateOperState,
}

/// Same as [`rtnetlink::packet_route::link::State`] but (de)serializable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkSpecStateOperState {
    Unknown,
    NotPresent,
    Down,
    LowerLayerDown,
    Testing,
    Dormant,
    Up,
    Other(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkSpecPlan {
    Create(LinkMessage),
    Modify(LinkMessage),
    Delete(u32),
    Nop,
}

#[derive(Error, Debug)]
pub enum LinkSpecReconcileError {
    #[error("netlink stream closed while more message were expected")]
    UnexpectedEndOfRtnlStream,

    #[error("error during rtnetlink query")]
    Rtnl(#[from] rtnetlink::Error),

    #[error("state was not correctly built while refreshing")]
    IncompleteStateDuringRefresh(#[from] LinkSpecStateBuilderError),
}

impl Specification for LinkSpec {
    type State = LinkSpecState;

    const SCHEMA: &str = ".containeros.net.link-spec";
}

impl State for LinkSpecState {}

impl From<LinkSpecReconcileError> for tonic::Status {
    fn from(value: LinkSpecReconcileError) -> Self {
        Self::from_error(value.into())
    }
}

impl Reconcilable for LinkSpec {
    type CurrentState = <Self as Specification>::State;
    type Data = Handle;
    type Error = LinkSpecReconcileError;
    type Output = ReconcileResourceResponse;
    type Plan = LinkSpecPlan;
    type Resource = UserConfigResource<Self>;

    async fn refresh(
        resource: &Self::Resource,
        data: &mut Self::Data,
    ) -> Result<Option<Self::CurrentState>, Self::Error> {
        let mut state = LinkSpecStateBuilder::default();

        let mut links = data
            .link()
            .get()
            .match_name(resource.id().name().clone())
            .execute();

        let link = links
            .next()
            .await
            .ok_or(LinkSpecReconcileError::UnexpectedEndOfRtnlStream)?;
        let link = match link {
            Ok(v) => v,
            Err(ref err @ rtnetlink::Error::NetlinkError(ref inner)) => {
                if let Some(code) = &inner.code
                    && Errno::from_raw_os_error(code.abs().into())
                        == Errno::NODEV
                {
                    return Ok(None);
                }

                return Err(LinkSpecReconcileError::from(err.clone()));
            }
            Err(err) => return Err(LinkSpecReconcileError::from(err)),
        };

        state.index(link.header.index);
        state.admin_up(link.header.flags.intersects(LinkFlags::Up));
        state.running(link.header.flags.intersects(LinkFlags::Running));
        for nla in link.attributes {
            match nla {
                LinkAttribute::OperState(link::State::Unknown) => {
                    state.oper_state(LinkSpecStateOperState::Unknown);
                }
                LinkAttribute::OperState(link::State::NotPresent) => {
                    state.oper_state(LinkSpecStateOperState::NotPresent);
                }
                LinkAttribute::OperState(link::State::Down) => {
                    state.oper_state(LinkSpecStateOperState::Down);
                }
                LinkAttribute::OperState(link::State::LowerLayerDown) => {
                    state.oper_state(LinkSpecStateOperState::LowerLayerDown);
                }
                LinkAttribute::OperState(link::State::Testing) => {
                    state.oper_state(LinkSpecStateOperState::Testing);
                }
                LinkAttribute::OperState(link::State::Dormant) => {
                    state.oper_state(LinkSpecStateOperState::Dormant);
                }
                LinkAttribute::OperState(link::State::Up) => {
                    state.oper_state(LinkSpecStateOperState::Up);
                }
                LinkAttribute::OperState(link::State::Other(v)) => {
                    state.oper_state(LinkSpecStateOperState::Other(v));
                }
                _ => {}
            }
        }

        state
            .build()
            .map(Some)
            .map_err(LinkSpecReconcileError::from)
    }

    fn plan(
        resource: &Self::Resource,
        data: &Self::Data,
        state: Option<&Self::CurrentState>,
    ) -> Result<Self::Plan, Self::Error> {
        if let ResourceSpec::Deleting(_) = resource.spec() {
            return match state {
                Some(s) => Ok(LinkSpecPlan::Delete(s.index)),
                None => Ok(LinkSpecPlan::Nop),
            };
        }

        let mut msg = LinkDummy::new(&resource.id().name());
        let empty = LinkDummy::new(&resource.id().name()).build();

        let desired_admin = &resource.spec_inner().admin_up;
        let current_admin = state.map_or(false, |state| state.admin_up);

        match (desired_admin, current_admin) {
            (true, false) => {
                msg = msg.up();
            }
            (false, true) => {
                msg = msg.down();
            }
            (true, true) | (false, false) => {}
        }

        let msg = msg.build();
        if msg == empty {
            return Ok(LinkSpecPlan::Nop);
        }

        match state {
            Some(_) => Ok(LinkSpecPlan::Modify(msg)),
            None => Ok(LinkSpecPlan::Create(msg)),
        }
    }

    async fn apply(
        resource: &Self::Resource,
        data: &mut Self::Data,
        state: Option<&Self::CurrentState>,
        plan: &Self::Plan,
    ) -> Result<Self::Output, Self::Error> {
        match plan {
            LinkSpecPlan::Create(msg) => {
                let apply = data
                    .link()
                    .add(msg.to_owned())
                    .execute()
                    .await
                    .map_err(LinkSpecReconcileError::from);

                let refresh = resource.refresh(data).await?;
                match (apply, refresh) {
                    (Ok(_), None) => todo!(),
                    (Ok(_), Some(state)) => Ok(ReconcileResourceResponse {
                        state: state.into_bytes().unwrap(),
                        ..Default::default()
                    }),
                    (Err(_), None) => todo!(),
                    (Err(_), Some(_)) => todo!(),
                }
            }

            LinkSpecPlan::Modify(msg) => {
                let apply = data
                    .link()
                    .change(msg.to_owned())
                    .execute()
                    .await
                    .map_err(LinkSpecReconcileError::from);

                let refresh = resource.refresh(data).await?;
                match (apply, refresh) {
                    (Ok(_), None) => todo!(),
                    (Ok(_), Some(state)) => Ok(ReconcileResourceResponse {
                        state: state.into_bytes().unwrap(),
                        ..Default::default()
                    }),
                    (Err(_), None) => todo!(),
                    (Err(_), Some(_)) => todo!(),
                }
            }

            LinkSpecPlan::Delete(index) => {
                let apply = data
                    .link()
                    .del(index.to_owned())
                    .execute()
                    .await
                    .map_err(LinkSpecReconcileError::from)
                    .map(|_| ReconcileResourceResponse {
                        deleted: vec![],
                        created: vec![],
                        updated: vec![],
                        state: vec![],
                    });

                let refresh = resource.refresh(data).await?;
                match (apply, refresh) {
                    (Ok(_), None) => Ok(ReconcileResourceResponse {
                        ..Default::default()
                    }),
                    (Ok(_), Some(state)) => Ok(ReconcileResourceResponse {
                        state: state.into_bytes().unwrap(),
                        ..Default::default()
                    }),
                    (Err(_), None) => todo!(),
                    (Err(_), Some(_)) => todo!(),
                }
            }

            LinkSpecPlan::Nop => {
                return Ok(ReconcileResourceResponse {
                    state: state.cloned().unwrap().into_bytes().unwrap(),
                    ..Default::default()
                });
            }
        }
    }
}
