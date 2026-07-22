use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::{Context as _, Result, bail};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    PrivateIdentity,
    Resource,
    ResourceResponse,
    Status,
    ValidateResponse,
};
use derive_builder::Builder;
use futures::{StreamExt as _, TryStreamExt as _};
use rtnetlink::packet_route::route::{
    RouteAddress,
    RouteAttribute,
    RouteMessage,
    RouteScope,
};
use rtnetlink::{Handle, RouteMessageBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RouteReconciler {
    rtnl: Handle,
}

pub type RouteResource = Resource<RouteSpec, RouteDerivedSpec, RouteState>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSpec {
    Ipv4 {
        destination: Ipv4Addr,
        prefix_len: u8,
        gateway: Ipv4Addr,
        parent: Option<String>,
    },
    Ipv6 {
        destination: Ipv6Addr,
        prefix_len: u8,
        gateway: Ipv6Addr,
        parent: Option<String>,
    },
}

type RouteDerivedSpec = ();

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(pattern = "mutable", vis = "pub(crate)")]
pub struct RouteState {}

#[derive(Debug)]
enum RouteContext {
    NoRoute,
    Route(RouteState),
}

#[derive(Debug, Clone)]
enum RoutePlan {
    Create(RouteMessage),
    Delete(RouteMessage),
    Noop,
}

impl RouteReconciler {
    #[must_use]
    pub const fn new_with(handle: Handle) -> Self {
        Self { rtnl: handle }
    }
}

// TODO: There should be a mecanisme to "Recreate" resources (+ with the
// possibility to create before destroy). The current implementation leaves
// "dangling" routees...
impl RouteReconciler {
    pub async fn validate(
        &self,
        _key: Key,
        spec: RouteSpec,
        resource: Option<RouteResource>,
    ) -> Result<ValidateResponse<RouteDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: (),
            children: vec![],
            dependencies: self.deps(spec.clone()).into_iter().collect(),
        })
    }

    #[must_use]
    pub fn deps(&self, spec: RouteSpec) -> Vec<Identity> {
        let parent = match spec {
            RouteSpec::Ipv4 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent,
            }
            | RouteSpec::Ipv6 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent,
            } => parent,
        };

        parent.map_or_else(std::vec::Vec::new, |parent| {
            vec![Identity::Private(PrivateIdentity::Dynamic(Key {
                schema: "network:address".to_owned(),
                name: Some(parent),
            }))]
        })
    }

    #[expect(clippy::too_many_lines, reason = "TODO")]
    pub async fn reconcile(
        &self,
        resource: RouteResource,
    ) -> Result<ResourceResponse<RouteState>> {
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: resource.state,
                children: vec![],
                dependencies: self
                    .deps(resource.spec.clone())
                    .into_iter()
                    .collect(),
            });
        }

        let cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: resource.state,
                    children: vec![],
                    dependencies: self
                        .deps(resource.spec.clone())
                        .into_iter()
                        .collect(),
                });
            }
        };

        let state = match &cx {
            RouteContext::NoRoute => None,
            RouteContext::Route(state) => Some(state.clone()),
        };

        let plan = match self.plan(&resource, cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: self
                        .deps(resource.spec.clone())
                        .into_iter()
                        .collect(),
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
                    dependencies: self
                        .deps(resource.spec.clone())
                        .into_iter()
                        .collect(),
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
                    dependencies: self
                        .deps(resource.spec.clone())
                        .into_iter()
                        .collect(),
                });
            }
        };

        let state = match &new_cx {
            RouteContext::NoRoute => None,
            RouteContext::Route(state) => Some(state.clone()),
        };

        let new_plan = match self.plan(&resource, new_cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state,
                    children: vec![],
                    dependencies: self
                        .deps(resource.spec.clone())
                        .into_iter()
                        .collect(),
                });
            }
        };

        let status = match new_plan {
            RoutePlan::Noop if matches!(resource.phase, Phase::Teardown) => {
                Status::Deleted
            }
            RoutePlan::Noop => Status::Ready,
            RoutePlan::Create(_) | RoutePlan::Delete(_) => Status::NotReady,
        };

        Ok(ResourceResponse {
            status,
            state,
            children: vec![],
            dependencies: self
                .deps(resource.spec.clone())
                .into_iter()
                .collect(),
        })
    }

    async fn validate_new_spec(&self, _spec: &RouteSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        resource: &RouteResource,
        spec: &RouteSpec,
    ) -> Result<()> {
        if spec != &resource.spec {
            bail!("cannot change route specification");
        }

        self.validate_new_spec(spec).await
    }

    #[expect(
        clippy::too_many_lines,
        reason = "rtnl messages are complex but splitting the function would \
                  be annoying"
    )]
    async fn refresh(&self, resource: &RouteResource) -> Result<RouteContext> {
        // NOTE: Trying to filter here by adding a gateway or destination breaks
        // everything for unknown reasons. Adding a destination seems to
        // "resolve" the route
        let msg = match resource.spec {
            RouteSpec::Ipv4 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent: _,
            } => RouteMessageBuilder::<Ipv4Addr>::new()
                .scope(RouteScope::Universe)
                .table_id(libc::RT_TABLE_MAIN.into())
                .build(),

            RouteSpec::Ipv6 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent: _,
            } => RouteMessageBuilder::<Ipv6Addr>::new()
                .scope(RouteScope::Universe)
                .table_id(libc::RT_TABLE_MAIN.into())
                .build(),
        };

        let default = match resource.spec {
            RouteSpec::Ipv4 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent: _,
            } => IpAddr::V4("0.0.0.0".parse()?),
            RouteSpec::Ipv6 {
                destination: _,
                prefix_len: _,
                gateway: _,
                parent: _,
            } => IpAddr::V6("::".parse()?),
        };

        let mut routes =
            self.rtnl.route().get(msg).execute().try_filter(|route| {
                let route_gw =
                    route.attributes.iter().find_map(|attr| match attr {
                        RouteAttribute::Gateway(RouteAddress::Inet(gw)) => {
                            Some(IpAddr::V4(*gw))
                        }
                        RouteAttribute::Gateway(RouteAddress::Inet6(gw)) => {
                            Some(IpAddr::V6(*gw))
                        }
                        _ => None,
                    });

                let route_destination =
                    route.attributes.iter().find_map(|attr| match attr {
                        RouteAttribute::Destination(RouteAddress::Inet(
                            dest,
                        )) => Some(IpAddr::V4(*dest)),
                        RouteAttribute::Destination(RouteAddress::Inet6(
                            dest,
                        )) => Some(IpAddr::V6(*dest)),
                        _ => None,
                    });

                let (destination, prefix_len, gateway) = match resource.spec {
                    RouteSpec::Ipv4 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => (
                        IpAddr::V4(destination),
                        prefix_len,
                        IpAddr::V4(gateway),
                    ),
                    RouteSpec::Ipv6 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => (
                        IpAddr::V6(destination),
                        prefix_len,
                        IpAddr::V6(gateway),
                    ),
                };

                let route_destination = match route_destination {
                    Some(r) => Some(r),
                    None if route.header.scope == RouteScope::Universe => {
                        Some(default)
                    }
                    None => None,
                };

                let is_matching_destination =
                    route_destination == Some(destination);

                let is_matching_len =
                    route.header.destination_prefix_length == prefix_len;

                // dbg!(route);
                // dbg!(&route_gw);
                // dbg!(&gateway);
                // dbg!(&route_destination);
                // dbg!(&destination);

                let is_matching_gw = route_gw == Some(gateway);
                let is_matching = is_matching_destination
                    && is_matching_len
                    && is_matching_gw;

                // dbg!(&is_matching_destination);
                // dbg!(&is_matching_gw);
                // dbg!(&is_matching_len);
                // dbg!(&is_matching);

                std::future::ready(is_matching)
            });

        let route = routes
            .try_next()
            .await
            .context("failed to retrieve routees")?;

        if routes.next().await.is_some() {
            bail!("found multiple routees while at most one was expected");
        }

        let Some(route) = route else {
            return Ok(RouteContext::NoRoute);
        };

        // dbg!(&route);

        RouteState::try_from_message(&route).map(RouteContext::Route)
    }

    async fn plan(
        &self,
        resource: &RouteResource,
        cx: RouteContext,
    ) -> Result<RoutePlan> {
        match (&resource.phase, cx) {
            (Phase::Teardown, RouteContext::Route(_)) => {
                let msg = match resource.spec {
                    RouteSpec::Ipv4 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => RouteMessageBuilder::<Ipv4Addr>::new()
                        .destination_prefix(destination, prefix_len)
                        .gateway(gateway)
                        .build(),
                    RouteSpec::Ipv6 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => RouteMessageBuilder::<Ipv6Addr>::new()
                        .destination_prefix(destination, prefix_len)
                        .gateway(gateway)
                        .build(),
                };

                Ok(RoutePlan::Delete(msg))
            }

            (Phase::Running, RouteContext::NoRoute) => {
                let msg = match resource.spec {
                    RouteSpec::Ipv4 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => RouteMessageBuilder::<Ipv4Addr>::new()
                        .destination_prefix(destination, prefix_len)
                        .gateway(gateway)
                        .build(),
                    RouteSpec::Ipv6 {
                        destination,
                        prefix_len,
                        gateway,
                        parent: _,
                    } => RouteMessageBuilder::<Ipv6Addr>::new()
                        .destination_prefix(destination, prefix_len)
                        .gateway(gateway)
                        .build(),
                };
                // dbg!(&msg);

                Ok(RoutePlan::Create(msg))
            }

            (Phase::Running, RouteContext::Route(_))
            | (
                Phase::Shutdown | Phase::Teardown,
                RouteContext::NoRoute | RouteContext::Route(_),
            ) => Ok(RoutePlan::Noop),
        }
    }

    async fn apply(
        &self,
        _resource: &RouteResource,
        plan: &RoutePlan,
    ) -> Result<()> {
        match plan {
            RoutePlan::Create(msg) => self
                .rtnl
                .route()
                .add(msg.clone())
                .execute()
                .await
                .context("unable to create route"),
            RoutePlan::Delete(msg) => self
                .rtnl
                .route()
                .del(msg.clone())
                .execute()
                .await
                .context("unable to delete route"),
            RoutePlan::Noop => Ok(()),
        }
    }
}

impl RouteState {
    fn try_from_message(_message: &RouteMessage) -> Result<Self> {
        let state = RouteStateBuilder::default();
        state.build().context("unable to build route state")
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
        use rtnetlink::sys::AsyncSocket as _;

        use super::*;

        async fn create_reconciler() -> (RouteReconciler, RouteResource) {
            create_reconciler_custom("10.0.2.15/24", "0.0.0.0/0", "10.0.2.2")
                .await
        }

        async fn create_reconciler_custom(
            addr: &str,
            dest: &str,
            gw: &str,
        ) -> (RouteReconciler, RouteResource) {
            let (mut conn, handle, _) =
                new_connection_with_socket::<SmolSocket>().unwrap();
            conn.socket_mut()
                .socket_mut()
                .set_netlink_get_strict_chk(true)
                .unwrap();
            smol::spawn(conn).detach();

            handle
                .link()
                .add(LinkDummy::new("dummy0").up().build())
                .execute()
                .await
                .unwrap();

            handle
                .address()
                .add(
                    2,
                    addr.split_once('/').unwrap().0.parse().unwrap(),
                    addr.split_once('/').unwrap().1.parse().unwrap(),
                )
                .execute()
                .await
                .unwrap();

            let reconciler = RouteReconciler::new_with(handle);

            let spec = RouteSpec::Ipv4 {
                destination: dest.split_once('/').unwrap().0.parse().unwrap(),
                prefix_len: dest.split_once('/').unwrap().1.parse().unwrap(),
                gateway: gw.parse().unwrap(),
                parent: None,
            };

            let addr = RouteResource {
                id: Identity::Private(PrivateIdentity::Static(Key {
                    schema: String::new(),
                    name: None,
                })),
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
        fn create_route_should_succeed() {
            let (reconciler, addr) = smol::block_on(create_reconciler());

            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let _ = result.state.unwrap();

            let count = smol::block_on(
                reconciler
                    .rtnl
                    .route()
                    .get(RouteMessageBuilder::<Ipv4Addr>::new().build())
                    .execute()
                    .count(),
            );

            // 0: the 10.0.0.0/24 added by setting an address
            // 1: the 10.0.0.2/32 added by setting an address
            // 2: the 10.0.0.255/32 added by setting an address
            // 3: *our* route
            assert_eq!(count, 1);
        }

        #[test]
        #[isolate]
        fn existing_route_should_succeed() {
            let (reconciler, mut addr) = smol::block_on(create_reconciler());

            let result =
                smol::block_on(reconciler.reconcile(addr.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            addr.status = Status::Unknown;
            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let _ = result.state.unwrap();

            let count = smol::block_on(
                reconciler
                    .rtnl
                    .route()
                    .get(RouteMessageBuilder::<Ipv4Addr>::new().build())
                    .execute()
                    .count(),
            );

            // See [`create_route_should_succeed`] for an explanation
            assert_eq!(count, 1);
        }

        #[test]
        #[isolate]
        fn non_aggregated_should_succeed() {
            let (reconciler, mut addr) =
                smol::block_on(create_reconciler_custom(
                    "10.0.2.15/24",
                    "192.168.0.0/24",
                    "10.0.2.2",
                ));

            smol::block_on(
                reconciler
                    .rtnl
                    .address()
                    .add(2, "0.0.0.0".parse().unwrap(), 0)
                    .execute(),
            )
            .unwrap();

            let result =
                smol::block_on(reconciler.reconcile(addr.clone())).unwrap();
            assert_matches!(result.status, Status::Ready);

            addr.status = Status::Unknown;
            let result = smol::block_on(reconciler.reconcile(addr)).unwrap();
            assert_matches!(result.status, Status::Ready);

            let _ = result.state.unwrap();

            let count = smol::block_on(
                reconciler
                    .rtnl
                    .route()
                    .get(RouteMessageBuilder::<Ipv4Addr>::new().build())
                    .execute()
                    .count(),
            );

            // See [`create_route_should_succeed`] for an explanation
            assert_eq!(count, 1);
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
                reconciler
                    .rtnl
                    .route()
                    .get(RouteMessageBuilder::<Ipv4Addr>::new().build())
                    .execute()
                    .count(),
            );

            // See [`create_route_should_succeed`] for an explanation
            assert_eq!(count, 0);
        }
    }
}
