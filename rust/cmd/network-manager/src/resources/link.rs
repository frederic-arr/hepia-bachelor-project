use cos_api_reconciler::ReconcileDynamicResourceRequest;
use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::Reconcilable;
use derive_builder::Builder;
use futures::StreamExt;
use rtnetlink::packet_route::link::{LinkAttribute, LinkFlags, LinkMessage};
use rtnetlink::{
    Handle,
    LinkDummy,
    LinkMessageBuilder,
    LinkUnspec,
    new_connection,
};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

pub struct Link;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkSpec {
    pub link_type: LinkType,
    pub admin_up: bool,
    pub mtu: Option<u32>,
    pub address: Option<[u8; 6]>,
    pub broadcast: Option<[u8; 6]>,
    pub altnames: Vec<String>,
    pub arp: bool,
    pub promiscuous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct LinkState {
    pub index: u32,
    pub running: bool,
    pub admin_up: bool,
    pub oper_state: LinkOperState,

    pub mtu: u32,
    pub address: [u8; 6],
    pub broadcast: [u8; 6],
    // pub altnames: Vec<String>,
    pub arp: bool,
    pub promiscuity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LinkType {
    Dummy,
    Ethernet,
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
    type Error = String;
    type Input = ReconcileDynamicResourceRequest<LinkSpec, LinkState>;
    type Output = v1::ReconcileDynamicResourceResponse;
    type Plan = LinkPlan;
    type State = Option<LinkState>;

    const SCHEMA: &'static str = "res#containeros::net::link";

    async fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> Result<Self::State, Self::Error> {
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
                    return Ok(None);
                }

                panic!("{err}");
            }
            Err(err) => panic!("{err}"),
        };

        state.index(link.header.index);
        state.admin_up(link.header.flags.intersects(LinkFlags::Up));
        state.running(link.header.flags.intersects(LinkFlags::Running));
        state.arp(!link.header.flags.contains(LinkFlags::Noarp));

        // dbg!(&link);
        for nla in link.attributes {
            use rtnetlink::packet_route::link;
            match nla {
                LinkAttribute::Promiscuity(promiscuity) => {
                    state.promiscuity(promiscuity);
                }
                LinkAttribute::Mtu(mtu) => {
                    state.mtu(mtu);
                }
                LinkAttribute::Address(addr) => {
                    state.address(addr.try_into().unwrap());
                }
                LinkAttribute::Broadcast(brd) => {
                    state.broadcast(brd.try_into().unwrap());
                }
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

        Ok(state.build().map(Some).unwrap())
    }

    fn plan(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>> {
        let plan = match input.spec.link_type {
            LinkType::Dummy => {
                let empty = LinkDummy::new(&input.name).build();
                let msg = Self::plan_unspec(
                    ctx,
                    input,
                    refreshed_state,
                    LinkDummy::new(&input.name),
                )
                .build();

                match refreshed_state {
                    Some(_) => LinkPlan::Modify(msg),
                    None => LinkPlan::Create(msg),
                }
            }
            LinkType::Ethernet => {
                let msg = Self::plan_unspec(
                    ctx,
                    input,
                    refreshed_state,
                    LinkUnspec::new_with_name(&input.name),
                )
                .build();

                match refreshed_state {
                    Some(_) => LinkPlan::Modify(msg),
                    None => LinkPlan::Noop,
                }
            }
        };

        std::future::ready(Ok(plan))
    }

    async fn apply(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Result<Self::Apply, Self::Error> {
        match plan {
            LinkPlan::Create(msg) => ctx
                .link()
                .add(msg.clone())
                .execute()
                .await
                .map_err(|e| format!("unable to create link: {e}")),
            LinkPlan::Modify(msg) => ctx
                .link()
                .change(msg.clone())
                .execute()
                .await
                .map_err(|e| format!("unable to modify link: {e}")),
            LinkPlan::Delete(index) => ctx
                .link()
                .del(*index)
                .execute()
                .await
                .map_err(|e| format!("unable to delete link: {e}")),
            LinkPlan::Noop => Ok(()),
        }
    }

    async fn update(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> Result<Self::Output, Self::Error> {
        let new_state = Self::refresh(ctx, input).await?;
        Ok(v1::ReconcileDynamicResourceResponse {
            state: rmp_serde::to_vec_named(&new_state).unwrap(),
            children: vec![],
        })
    }
}

impl Link {
    fn plan_unspec<T>(
        ctx: &mut <Self as Reconcilable>::Context,
        input: &<Self as Reconcilable>::Input,
        refreshed_state: &<Self as Reconcilable>::State,
        mut msg: LinkMessageBuilder<T>,
    ) -> LinkMessageBuilder<T> {
        msg = match input.spec.admin_up {
            true => msg.up(),
            false => msg.down(),
        };

        if let Some(addr) = input.spec.address {
            msg = msg.address(addr.into())
        }

        if let Some(brd) = input.spec.broadcast {
            msg = msg
                .append_extra_attribute(LinkAttribute::Broadcast(brd.into()));
        }

        if let Some(mtu) = input.spec.mtu {
            msg = msg.mtu(mtu);
        }

        msg = msg.promiscuous(input.spec.promiscuous);
        msg = msg.arp(input.spec.arp);
        msg
    }
}
