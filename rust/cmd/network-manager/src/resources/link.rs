use cos_api_reconciler::ReconcileDynamicResourceRequest;
use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::Reconcilable;
use derive_builder::Builder;
use futures::StreamExt;
use rtnetlink::packet_route::link::{LinkAttribute, LinkFlags, LinkMessage};
use rtnetlink::{Handle, LinkDummy, new_connection};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

pub struct Link;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkSpec {
    pub admin_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct LinkState {
    pub index: u32,
    pub running: bool,
    pub admin_up: bool,
    pub oper_state: LinkOperState,
}

/// Same as [`rtnetlink::packet_route::link::State`] but (de)serializable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkOperState {
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
pub enum LinkPlan {
    Create(LinkMessage),
    Modify(LinkMessage),
    Delete(u32),
    Noop,
}

impl Reconcilable for Link {
    type Apply = ();
    type Context = Handle;
    type Input = ReconcileDynamicResourceRequest<LinkSpec, LinkState>;
    type Output = v1::ReconcileDynamicResourceResponse;
    type Plan = LinkPlan;
    type State = Option<LinkState>;

    const SCHEMA: &'static str = "res#containeros::net::link";

    async fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> Self::State {
        let mut state = LinkStateBuilder::default();

        let mut links =
            ctx.link().get().match_name(input.name.clone()).execute();

        let link = links.next().await.expect("at least one RTNL message");
        assert!(
            links.next().await.is_none(),
            "got multiple links while only one was expected"
        );

        let link = match link {
            Ok(v) => v,
            Err(rtnetlink::Error::NetlinkError(err)) => {
                if let Some(code) = err.code
                    && Errno::from_raw_os_error(code.abs().into())
                        == Errno::NODEV
                {
                    return None;
                }

                panic!("{err}");
            }
            Err(err) => panic!("{err}"),
        };

        state.index(link.header.index);
        state.admin_up(link.header.flags.intersects(LinkFlags::Up));
        state.running(link.header.flags.intersects(LinkFlags::Running));
        for nla in link.attributes {
            use rtnetlink::packet_route::link;
            match nla {
                LinkAttribute::OperState(link::State::Up) => {
                    state.oper_state(LinkOperState::Up);
                }
                LinkAttribute::OperState(link::State::Down) => {
                    state.oper_state(LinkOperState::Down);
                }
                LinkAttribute::OperState(_) => {
                    state.oper_state(LinkOperState::Unknown);
                }
                _ => {}
            }
        }

        state.build().map(Some).unwrap()
    }

    fn plan(
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Self::Plan> {
        let mut msg = LinkDummy::new(&input.name);
        let empty = LinkDummy::new(&input.name).build();

        let msg = match (
            input.spec.admin_up,
            refreshed_state.as_ref().map(|s| s.admin_up),
        ) {
            (true, Some(true)) | (false, Some(false)) => msg,
            (true, _) => msg.up(),
            (false, _) => msg.down(),
        };

        let msg = msg.build();
        if msg == empty {
            return std::future::ready(LinkPlan::Noop);
        }

        let state = match refreshed_state {
            Some(_) => LinkPlan::Modify(msg),
            None => LinkPlan::Create(msg),
        };

        std::future::ready(state)
    }

    async fn apply(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Self::Apply {
        // let (conn, mut rtnl, _) = new_connection().unwrap();
        // let task = tokio::spawn(conn);

        match plan {
            LinkPlan::Create(msg) => {
                ctx.link().add(msg.clone()).execute().await.unwrap();
            }
            LinkPlan::Modify(msg) => {
                ctx.link().change(msg.clone()).execute().await.unwrap();
            }
            LinkPlan::Delete(index) => {
                ctx.link().del(*index).execute().await.unwrap();
            }
            LinkPlan::Noop => (),
        }
    }

    async fn update(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> Self::Output {
        let new_state = Self::refresh(ctx, input).await.unwrap();
        v1::ReconcileDynamicResourceResponse {
            state: rmp_serde::to_vec_named(&new_state).unwrap(),
            children: vec![],
        }
    }
}
