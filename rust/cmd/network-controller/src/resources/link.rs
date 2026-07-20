use std::collections::HashSet;

use anyhow::{Context as _, Result, anyhow, bail};
use cos_proto_reconciler::{
    Phase,
    Resource,
    ResourceResponse,
    Status,
    ValidateResponse,
};
use derive_builder::Builder;
use futures::StreamExt as _;
use rtnetlink::packet_core::ErrorMessage;
use rtnetlink::packet_route::link::LinkMessage;
use rtnetlink::{
    Handle,
    LinkDummy,
    LinkMessageBuilder,
    LinkUnspec,
    packet_route,
};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LinkReconciler {
    rtnl: Handle,
}

pub type LinkResource = Resource<LinkSpec, LinkDerivedSpec, LinkState>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSpec {
    pub name: String,
    pub admin_up: bool,
    pub link_type: LinkSpecType,
}

type LinkDerivedSpec = ();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkSpecType {
    Dummy(LinkSpecDummy),
    Unspec(LinkSpecUnspec),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSpecDummy {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSpecUnspec {}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(pattern = "mutable", vis = "pub(crate)")]
pub struct LinkState {
    pub index: u32,
    pub running: bool,
    pub admin_up: bool,
    pub oper_state: LinkOperState,

    // #[builder(default)]
    pub link_type: LinkStateType,

    pub mtu: u32,
    pub address: [u8; 6],
    pub broadcast: [u8; 6],

    #[builder(default)]
    pub alt_names: Vec<String>,
    pub arp: bool,
    pub promiscuity: u32,
}

/// Same as [`rtnetlink::packet_route::link::State`] but (de)serializable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkOperState {
    Unknown,
    NotPresent,
    Down,
    LowerLayerDown,
    Testing,
    Dormant,
    Up,
    Other(u8),
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkStateType {
    Dummy(LinkStateDummy),
    Unspec(LinkStateUnspec),
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStateDummy {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStateUnspec {}

#[derive(Debug)]
enum LinkContext {
    NoLink,
    Link(LinkState),
}

#[derive(Debug, Clone)]
enum LinkPlan {
    Create(LinkMessage),
    Update(LinkMessage),
    Delete(LinkState),
    Noop,
}

impl Default for LinkSpecType {
    fn default() -> Self {
        Self::Unspec(LinkSpecUnspec {})
    }
}

impl Default for LinkStateType {
    fn default() -> Self {
        Self::Unspec(LinkStateUnspec {})
    }
}

impl LinkSpecType {
    #[must_use]
    pub fn is_physical(&self) -> bool {
        matches!(self, Self::Unspec(_))
    }
}

impl LinkStateType {
    #[must_use]
    pub fn is_physical(&self) -> bool {
        matches!(self, Self::Unspec(_))
    }
}

impl LinkReconciler {
    #[must_use]
    pub const fn new_with(handle: Handle) -> Self {
        Self { rtnl: handle }
    }
}

impl LinkReconciler {
    pub async fn validate(
        &self,
        spec: LinkSpec,
        resource: Option<LinkResource>,
    ) -> Result<ValidateResponse<LinkDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: (),
            children: vec![],
            dependencies: vec![],
        })
    }

    pub async fn reconcile(
        &self,
        resource: LinkResource,
    ) -> Result<ResourceResponse<LinkState>> {
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: resource.state,
                children: vec![],
                dependencies: HashSet::new(),
            });
        }

        let cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: resource.state,
                    children: vec![],
                    dependencies: HashSet::new(),
                });
            }
        };

        let state = match &cx {
            LinkContext::NoLink => None,
            LinkContext::Link(state) => Some(state.clone()),
        };

        let plan = match self.plan(&resource, cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::new(),
                });
            }
        };

        let () = match self.apply(&resource, &plan).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::new(),
                });
            }
        };

        let new_cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::new(),
                });
            }
        };

        let state = match &new_cx {
            LinkContext::NoLink => None,
            LinkContext::Link(state) => Some(state.clone()),
        };

        let new_plan = match self.plan(&resource, new_cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::new(),
                });
            }
        };

        let status = match new_plan {
            LinkPlan::Noop if matches!(resource.phase, Phase::Teardown) => {
                Status::Deleted
            }
            LinkPlan::Noop => Status::Ready,
            LinkPlan::Create(_) | LinkPlan::Update(_) | LinkPlan::Delete(_) => {
                Status::NotReady
            }
        };

        Ok(ResourceResponse {
            status,
            state,
            children: vec![],
            dependencies: HashSet::new(),
        })
    }

    async fn validate_new_spec(&self, _spec: &LinkSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &LinkResource,
        spec: &LinkSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    async fn refresh(&self, resource: &LinkResource) -> Result<LinkContext> {
        let mut links = self
            .rtnl
            .link()
            .get()
            .match_name(resource.spec.name.clone())
            .execute();

        let link = links
            .next()
            .await
            .ok_or_else(|| anyhow!("no messages receieved"))
            .and_then(nodev_as_none)
            .context("failed to retrieve links")?;

        if links.next().await.is_some() {
            bail!("found multiple links while at most one was expected");
        }

        let Some(link) = link else {
            return Ok(LinkContext::NoLink);
        };

        LinkState::try_from_message(&link).map(LinkContext::Link)
    }

    async fn plan(
        &self,
        resource: &LinkResource,
        cx: LinkContext,
    ) -> Result<LinkPlan> {
        let spec = &resource.spec;
        match (&resource.phase, cx) {
            (Phase::Teardown, LinkContext::Link(link)) => {
                Ok(LinkPlan::Delete(link))
            }

            (Phase::Running, LinkContext::NoLink) => {
                let msg = match spec.link_type {
                    LinkSpecType::Dummy(_) => {
                        let msg = Self::link_plan_unspec(
                            spec,
                            None,
                            LinkDummy::new(&spec.name),
                        );
                        msg.build()
                    }
                    LinkSpecType::Unspec(_) => {
                        let msg = Self::link_plan_unspec(
                            spec,
                            None,
                            LinkUnspec::new_with_name(&spec.name),
                        );
                        msg.build()
                    }
                };

                Ok(LinkPlan::Create(msg))
            }

            (Phase::Running, LinkContext::Link(link)) => {
                let (empty, msg) = match spec.link_type {
                    LinkSpecType::Dummy(_) => {
                        let empty = LinkDummy::new(&spec.name).build();
                        let msg = Self::link_plan_unspec(
                            spec,
                            Some(&link),
                            LinkDummy::new(&spec.name),
                        );
                        (empty, msg.build())
                    }
                    LinkSpecType::Unspec(_) => {
                        let empty =
                            LinkUnspec::new_with_name(&spec.name).build();
                        let msg = Self::link_plan_unspec(
                            spec,
                            Some(&link),
                            LinkUnspec::new_with_name(&spec.name),
                        );
                        (empty, msg.build())
                    }
                };

                if msg == empty {
                    return Ok(LinkPlan::Noop);
                }

                Ok(LinkPlan::Update(msg))
            }
            (
                Phase::Shutdown | Phase::Teardown,
                LinkContext::NoLink | LinkContext::Link(_),
            ) => Ok(LinkPlan::Noop),
        }
    }

    async fn apply(
        &self,
        _resource: &LinkResource,
        plan: &LinkPlan,
    ) -> Result<()> {
        match plan {
            LinkPlan::Create(msg) => self
                .rtnl
                .link()
                .add(msg.clone())
                .execute()
                .await
                .context("unable to create link"),
            LinkPlan::Update(msg) => self
                .rtnl
                .link()
                .change(msg.clone())
                .execute()
                .await
                .context("unable to modify link"),
            LinkPlan::Delete(state) => self
                .rtnl
                .link()
                .del(state.index)
                .execute()
                .await
                .context("unable to delete link"),
            LinkPlan::Noop => Ok(()),
        }
    }
}

fn nodev_as_none(
    res: Result<LinkMessage, rtnetlink::Error>,
) -> anyhow::Result<Option<LinkMessage>> {
    let err = match res {
        Ok(v) => return Ok(Some(v)),
        Err(err) => err,
    };

    let rtnetlink::Error::NetlinkError(msg) = &err else {
        return Err(err.into());
    };

    let Some(code) = msg.code else {
        return Err(err.into());
    };

    // Returned code is negative but Errno is positive
    if code.get().abs() != Errno::NODEV.raw_os_error() {
        return Err(err.into());
    }

    Ok(None)
}

impl From<packet_route::link::State> for LinkOperState {
    fn from(value: packet_route::link::State) -> Self {
        use packet_route::link::State;

        match value {
            State::Unknown => Self::Unknown,
            State::NotPresent => Self::NotPresent,
            State::Down => Self::Down,
            State::LowerLayerDown => Self::LowerLayerDown,
            State::Testing => Self::Testing,
            State::Dormant => Self::Dormant,
            State::Up => Self::Up,
            State::Other(v) => Self::Other(v),
            _ => Self::Unsupported,
        }
    }
}

impl LinkState {
    fn try_from_message(message: &LinkMessage) -> Result<Self> {
        let mut state = LinkStateBuilder::default();
        state.add_from_header(&message.header);
        state.try_add_from_attributes(&message.attributes)?;

        state.build().context("unable to build link state")
    }
}

impl LinkStateBuilder {
    fn add_from_header(&mut self, header: &packet_route::link::LinkHeader) {
        use rtnetlink::packet_route::link::LinkFlags;

        self.index(header.index);
        self.admin_up(header.flags.intersects(LinkFlags::Up));
        self.running(header.flags.intersects(LinkFlags::Running));
        self.arp(!header.flags.contains(LinkFlags::Noarp));

        // TODO: Add AddressFamiliy
        // TODO: Add LinkLayerType
        // TODO: Add all LinkFlags
    }

    fn try_add_from_attributes(
        &mut self,
        attributes: &[packet_route::link::LinkAttribute],
    ) -> Result<()> {
        use rtnetlink::packet_route::link::LinkAttribute;

        for attr in attributes {
            match attr {
                LinkAttribute::LinkInfo(infos) => {
                    self.try_add_from_infos(infos)?;
                }
                LinkAttribute::PropList(props) => {
                    self.try_add_from_props(props)?;
                }
                LinkAttribute::Promiscuity(promiscuity) => {
                    self.promiscuity(*promiscuity);
                }
                LinkAttribute::Mtu(mtu) => {
                    self.mtu(*mtu);
                }
                LinkAttribute::Address(addr) => {
                    let Ok(addr) = <[u8; 6]>::try_from(addr.as_slice()) else {
                        bail!("unable to convert link hardware address");
                    };

                    self.address(addr);
                }
                LinkAttribute::Broadcast(brd) => {
                    let Ok(brd) = <[u8; 6]>::try_from(brd.as_slice()) else {
                        bail!("unable to convert link broadcast address");
                    };

                    self.broadcast(brd);
                }
                LinkAttribute::OperState(oper_state) => {
                    self.oper_state((*oper_state).into());
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn try_add_from_props(
        &mut self,
        props: &[packet_route::link::Prop],
    ) -> Result<()> {
        use packet_route::link::Prop;

        let alt_names = props
            .iter()
            .filter_map(|prop| match prop {
                Prop::AltIfName(alt_if_name) => Some(alt_if_name.clone()),
                _ => None,
            })
            .collect();

        self.alt_names(alt_names);
        Ok(())
    }

    fn try_add_from_infos(
        &mut self,
        infos: &[packet_route::link::LinkInfo],
    ) -> Result<()> {
        use packet_route::link::{InfoKind, LinkInfo};

        let kind = infos.iter().find_map(|info| match info {
            LinkInfo::Kind(kind) => Some(kind),
            _ => None,
        });

        let data = infos.iter().find_map(|info| match info {
            LinkInfo::Data(data) => Some(data),
            _ => None,
        });

        let link_type = match (kind, data) {
            (None, None) => LinkStateType::Unspec(LinkStateUnspec {}),
            (Some(kind), None) => match kind {
                InfoKind::Dummy => LinkStateType::Dummy(LinkStateDummy {}),
                kind => LinkStateType::Unsupported(kind.to_string()),
            },
            (Some(kind), Some(_)) => {
                LinkStateType::Unsupported(kind.to_string())
            }
            (None, Some(_)) => bail!("got link data without link kind"),
        };

        self.link_type(link_type);
        Ok(())
    }
}

impl LinkReconciler {
    fn link_plan_unspec<T>(
        spec: &LinkSpec,
        current: Option<&LinkState>,
        mut msg: LinkMessageBuilder<T>,
    ) -> LinkMessageBuilder<T> {
        if current.is_none_or(|v| v.admin_up != spec.admin_up) {
            if spec.admin_up {
                msg = msg.up();
            } else {
                msg = msg.down();
            }
        }

        msg
    }
}

fn is_errno(err: &rtnetlink::Error, errno: Errno) -> bool {
    use rtnetlink::Error;

    let Error::NetlinkError(ErrorMessage {
        code: Some(code), ..
    }) = err
    else {
        return false;
    };

    Errno::from_raw_os_error(code.abs().into()) == errno
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use isolation::isolate;
    use rtnetlink::new_connection_with_socket;
    use rtnetlink::sys::SmolSocket;

    use super::*;

    mod reconciliation {
        use cos_proto_reconciler::{Identity, Key};

        use super::*;

        fn create_reconciler() -> (LinkReconciler, LinkResource) {
            let (conn, handle, _) =
                new_connection_with_socket::<SmolSocket>().unwrap();
            smol::spawn(conn).detach();
            let reconciler = LinkReconciler::new_with(handle);

            let spec = LinkSpec {
                name: "dummy0".to_owned(),
                admin_up: true,
                link_type: LinkSpecType::Dummy(LinkSpecDummy {}),
            };

            let link = LinkResource {
                id: Identity::Static(Key {
                    schema: String::new(),
                    name: None,
                }),
                phase: Phase::Running,
                status: Status::Unknown,
                spec,
                derived_spec: (),
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            };

            (reconciler, link)
        }

        #[test]
        #[isolate]
        fn create_dummy_should_succeed() {
            let (reconciler, link) = create_reconciler();

            let result = smol::block_on(reconciler.reconcile(link)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let state = result.state.unwrap();

            assert_eq!(state.index, 2);
            assert!(state.admin_up);
            assert_eq!(state.oper_state, LinkOperState::Unknown);

            let count =
                smol::block_on(reconciler.rtnl.link().get().execute().count());
            assert_eq!(count, 2);
        }

        #[test]
        #[isolate]
        fn existing_dummy_should_succeed() {
            let (reconciler, mut link) = create_reconciler();

            let result =
                smol::block_on(reconciler.reconcile(link.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            link.status = Status::Unknown;
            let result = smol::block_on(reconciler.reconcile(link)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let state = result.state.unwrap();
            assert_eq!(state.index, 2);
            assert!(state.admin_up);
            assert_eq!(state.oper_state, LinkOperState::Unknown);

            let count =
                smol::block_on(reconciler.rtnl.link().get().execute().count());
            assert_eq!(count, 2);
        }

        #[test]
        #[isolate]
        fn updating_dummy_should_succeed() {
            let (reconciler, mut link) = create_reconciler();

            let result =
                smol::block_on(reconciler.reconcile(link.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            link.spec.admin_up = false;
            let result = smol::block_on(reconciler.reconcile(link)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let state = result.state.unwrap();
            assert_eq!(state.index, 2);
            assert!(!state.admin_up);
            assert_eq!(state.oper_state, LinkOperState::Down);

            let count =
                smol::block_on(reconciler.rtnl.link().get().execute().count());
            assert_eq!(count, 2);
        }

        #[test]
        #[isolate]
        fn delete_dummy_should_succeed() {
            let (reconciler, mut link) = create_reconciler();

            let result =
                smol::block_on(reconciler.reconcile(link.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            link.phase = Phase::Teardown;
            let result = smol::block_on(reconciler.reconcile(link)).unwrap();
            assert_matches!(result.status, Status::Deleted);
            assert_matches!(result.state, None);

            let count =
                smol::block_on(reconciler.rtnl.link().get().execute().count());
            assert_eq!(count, 1);
        }
    }
}
