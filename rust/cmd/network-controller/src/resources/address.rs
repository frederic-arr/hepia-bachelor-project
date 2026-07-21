use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::{Context as _, Result, bail};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    Resource,
    ResourceResponse,
    Status,
    ValidateResponse,
};
use derive_builder::Builder;
use futures::{StreamExt as _, TryStreamExt as _};
use rtnetlink::packet_route::address::AddressMessage;
use rtnetlink::{AddressMessageBuilder, Handle, packet_route};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

use crate::{LinkContext, LinkReconciler};

#[derive(Debug, Clone)]
pub struct AddressReconciler {
    rtnl: Handle,
}

pub type AddressResource =
    Resource<AddressSpec, AddressDerivedSpec, AddressState>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressSpec {
    pub dev: String,
    pub address: IpAddr,
    pub prefix_len: u8,
}

type AddressDerivedSpec = ();

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(pattern = "mutable", vis = "pub(crate)")]
pub struct AddressState {
    pub index: u32,
}

#[derive(Debug)]
enum AddressContext {
    NoAddress {
        link_index: u32,
    },
    Address {
        link_index: u32,
        state: AddressState,
    },
}

#[derive(Debug, Clone)]
enum AddressPlan {
    Create(u32),
    Delete(AddressMessage),
    Noop,
}

impl AddressReconciler {
    #[must_use]
    pub const fn new_with(handle: Handle) -> Self {
        Self { rtnl: handle }
    }
}

// TODO: There should be a mecanisme to "Recreate" resources (+ with the
// possibility to create before destroy). The current implementation leaves
// "dangling" addresses...
impl AddressReconciler {
    pub async fn validate(
        &self,
        spec: AddressSpec,
        resource: Option<AddressResource>,
    ) -> Result<ValidateResponse<AddressDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: (),
            children: vec![],
            dependencies: HashSet::from(self.deps(spec)),
        })
    }

    #[must_use]
    pub fn deps(&self, spec: AddressSpec) -> [Identity; 1] {
        [Identity::Dynamic(Key {
            schema: "network:link".to_owned(),
            name: Some(spec.dev),
        })]
    }

    pub async fn reconcile(
        &self,
        resource: AddressResource,
    ) -> Result<ResourceResponse<AddressState>> {
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: resource.state,
                children: vec![],
                dependencies: HashSet::from(self.deps(resource.spec)),
            });
        }

        let cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: resource.state,
                    children: vec![],
                    dependencies: HashSet::from(self.deps(resource.spec)),
                });
            }
        };

        let state = match &cx {
            AddressContext::NoAddress { link_index: _ } => None,
            AddressContext::Address {
                link_index: _,
                state,
            } => Some(state.clone()),
        };

        let plan = match self.plan(&resource, cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::from(self.deps(resource.spec)),
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
                    dependencies: HashSet::from(self.deps(resource.spec)),
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
                    dependencies: HashSet::from(self.deps(resource.spec)),
                });
            }
        };

        let state = match &new_cx {
            AddressContext::NoAddress { link_index: _ } => None,
            AddressContext::Address {
                link_index: _,
                state,
            } => Some(state.clone()),
        };

        let new_plan = match self.plan(&resource, new_cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: HashSet::from(self.deps(resource.spec)),
                });
            }
        };

        let status = match new_plan {
            AddressPlan::Noop if matches!(resource.phase, Phase::Teardown) => {
                Status::Deleted
            }
            AddressPlan::Noop => Status::Done,
            AddressPlan::Create(_) | AddressPlan::Delete(_) => Status::NotReady,
        };

        Ok(ResourceResponse {
            status,
            state,
            children: vec![],
            dependencies: HashSet::from(self.deps(resource.spec)),
        })
    }

    async fn validate_new_spec(&self, _spec: &AddressSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        resource: &AddressResource,
        spec: &AddressSpec,
    ) -> Result<()> {
        if spec != &resource.spec {
            bail!("cannot change address specification");
        }

        self.validate_new_spec(spec).await
    }

    async fn refresh(
        &self,
        resource: &AddressResource,
    ) -> Result<AddressContext> {
        let link_cx = LinkReconciler::get_link_info(
            &self.rtnl,
            resource.spec.dev.clone(),
        )
        .await?;

        let LinkContext::Link(link) = link_cx else {
            bail!("no link");
        };

        let mut addresses = self
            .rtnl
            .address()
            .get()
            .set_address_filter(resource.spec.address)
            .set_prefix_length_filter(resource.spec.prefix_len)
            .set_link_index_filter(link.index)
            .execute();

        let address = addresses
            .try_next()
            .await
            .context("failed to retrieve addresses")?;

        if addresses.next().await.is_some() {
            bail!("found multiple addresses while at most one was expected");
        }

        let Some(address) = address else {
            return Ok(AddressContext::NoAddress {
                link_index: link.index,
            });
        };

        AddressState::try_from_message(&address).map(|v| {
            AddressContext::Address {
                link_index: link.index,
                state: v,
            }
        })
    }

    async fn plan(
        &self,
        resource: &AddressResource,
        cx: AddressContext,
    ) -> Result<AddressPlan> {
        match (&resource.phase, cx) {
            (
                Phase::Teardown,
                AddressContext::Address {
                    link_index: _,
                    state,
                },
            ) => {
                let msg = match resource.spec.address {
                    IpAddr::V4(address) => {
                        AddressMessageBuilder::<Ipv4Addr>::new()
                            .index(state.index)
                            .address(address, resource.spec.prefix_len)
                            .build()
                    }
                    IpAddr::V6(address) => {
                        AddressMessageBuilder::<Ipv6Addr>::new()
                            .index(state.index)
                            .address(address, resource.spec.prefix_len)
                            .build()
                    }
                };

                Ok(AddressPlan::Delete(msg))
            }

            (Phase::Running, AddressContext::NoAddress { link_index }) => {
                Ok(AddressPlan::Create(link_index))
            }

            (
                Phase::Running | Phase::Shutdown | Phase::Teardown,
                AddressContext::NoAddress { link_index: _ }
                | AddressContext::Address {
                    link_index: _,
                    state: _,
                },
            ) => Ok(AddressPlan::Noop),
        }
    }

    async fn apply(
        &self,
        resource: &AddressResource,
        plan: &AddressPlan,
    ) -> Result<()> {
        match plan {
            AddressPlan::Create(link_index) => self
                .rtnl
                .address()
                .add(
                    *link_index,
                    resource.spec.address,
                    resource.spec.prefix_len,
                )
                .execute()
                .await
                .context("unable to create address"),
            AddressPlan::Delete(msg) => self
                .rtnl
                .address()
                .del(msg.clone())
                .execute()
                .await
                .context("unable to delete address"),
            AddressPlan::Noop => Ok(()),
        }
    }
}

fn nodev_as_none(
    res: Result<AddressMessage, rtnetlink::Error>,
) -> anyhow::Result<Option<AddressMessage>> {
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

impl AddressState {
    fn try_from_message(message: &AddressMessage) -> Result<Self> {
        let mut state = AddressStateBuilder::default();
        state.add_from_header(&message.header);

        state.build().context("unable to build address state")
    }
}

impl AddressStateBuilder {
    fn add_from_header(
        &mut self,
        header: &packet_route::address::AddressHeader,
    ) {
        self.index(header.index);
    }
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
        use rtnetlink::LinkDummy;

        use super::*;

        async fn create_reconciler() -> (AddressReconciler, AddressResource) {
            let (conn, handle, _) =
                new_connection_with_socket::<SmolSocket>().unwrap();
            smol::spawn(conn).detach();

            handle
                .link()
                .add(LinkDummy::new("dummy0").up().build())
                .execute()
                .await
                .unwrap();

            let reconciler = AddressReconciler::new_with(handle);

            let spec = AddressSpec {
                dev: "dummy0".to_owned(),
                address: "10.0.0.2".parse().unwrap(),
                prefix_len: 24,
            };

            let addr = AddressResource {
                id: Identity::Dynamic(Key {
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

            (reconciler, addr)
        }

        #[test]
        #[isolate]
        fn create_address_should_succeed() {
            let (reconciler, addr) = smol::block_on(create_reconciler());

            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let state = result.state.unwrap();
            assert_eq!(state.index, 2);

            let count = smol::block_on(
                reconciler.rtnl.address().get().execute().count(),
            );

            // 0: IPv6 LLA
            // 1: *our* address
            assert_eq!(count, 2);
        }

        #[test]
        #[isolate]
        fn existing_address_should_succeed() {
            let (reconciler, mut addr) = smol::block_on(create_reconciler());

            let result =
                smol::block_on(reconciler.reconcile(addr.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            addr.status = Status::Unknown;
            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let state = result.state.unwrap();
            assert_eq!(state.index, 2);

            let count = smol::block_on(
                reconciler.rtnl.address().get().execute().count(),
            );

            // See [`create_address_should_succeed`] for an explanation
            assert_eq!(count, 2);
        }

        #[test]
        #[isolate]
        fn delete_dummy_should_succeed() {
            let (reconciler, mut addr) = smol::block_on(create_reconciler());

            let result =
                smol::block_on(reconciler.reconcile(addr.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            addr.phase = Phase::Teardown;
            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Deleted);
            assert_matches!(result.state, None);

            let count = smol::block_on(
                reconciler.rtnl.address().get().execute().count(),
            );

            // See [`create_address_should_succeed`] for an explanation
            assert_eq!(count, 1);
        }
    }
}
