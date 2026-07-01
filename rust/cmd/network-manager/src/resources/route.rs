use std::net::{IpAddr, Ipv4Addr};

use cos_api_reconciler::ReconcileDynamicResourceRequest;
use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::Reconcilable;
use derive_builder::Builder;
use futures::{StreamExt, TryStreamExt};
use rtnetlink::packet_route::address::{AddressAttribute, AddressMessage};
use rtnetlink::packet_route::link::{LinkAttribute, LinkFlags, LinkMessage};
use rtnetlink::packet_route::route::{
    RouteAddress,
    RouteAttribute,
    RouteMessage,
};
use rtnetlink::{
    AddressMessageBuilder,
    Handle,
    LinkDummy,
    LinkMessageBuilder,
    LinkUnspec,
    RouteMessageBuilder,
    new_connection,
};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

pub struct Route;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RouteSpec {
    pub gateway: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub prefix_len: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct RouteState {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutePlan {
    Create(RouteMessage),
    Delete(u32),
    Noop,
}

impl Reconcilable for Route {
    type Apply = ();
    type Context = Handle;
    type Error = String;
    type Input = ReconcileDynamicResourceRequest<RouteSpec, RouteState>;
    type Output = v1::ReconcileDynamicResourceResponse;
    type Plan = RoutePlan;
    type State = Option<RouteState>;

    const SCHEMA: &'static str = "res#containeros::net::route";

    async fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> Result<Self::State, Self::Error> {
        let mut state = RouteStateBuilder::default();

        // getroute cannot filter on gateway
        // https://docs.kernel.org/netlink/specs/rt-route.html#getroute
        let query = RouteMessageBuilder::<Ipv4Addr>::new()
            // .destination_prefix(input.spec.destination, input.spec.prefix_len)
            .build();

        let mut routes = ctx.route().get(query).execute().try_filter(|route| {
            let route_gw =
                route.attributes.iter().find_map(|attr| match attr {
                    RouteAttribute::Gateway(RouteAddress::Inet(gw)) => Some(gw),
                    _ => None,
                });

            let route_destination =
                route.attributes.iter().find_map(|attr| match attr {
                    RouteAttribute::Destination(RouteAddress::Inet(dest)) => {
                        Some(dest)
                    }
                    _ => None,
                });

            let is_matching_destination =
                route_destination == Some(&input.spec.destination);

            let is_matching_len =
                route.header.destination_prefix_length == input.spec.prefix_len;

            let is_matching_gw = route_gw == Some(&input.spec.gateway);
            let is_matching =
                is_matching_destination && is_matching_len && is_matching_gw;

            std::future::ready(is_matching)
        });

        let route = routes
            .try_next()
            .await
            .map_err(|e| format!("unable to fetch routes: {e}"))?;

        if let Some(route) = routes.next().await {
            return Err(
                "got multiple routes while expected at moste one".to_string()
            );
        }

        let Some(route) = route else {
            return Ok(None);
        };

        Ok(state.build().map(Some).unwrap())
    }

    fn plan(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>> {
        let msg = RouteMessageBuilder::<Ipv4Addr>::new()
            .destination_prefix(input.spec.destination, input.spec.prefix_len)
            .gateway(input.spec.gateway)
            .build();

        let plan = match (&input.state, refreshed_state) {
            (None, None) => RoutePlan::Create(msg),
            _ => RoutePlan::Noop,
        };

        std::future::ready(Ok(plan))
    }

    async fn apply(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Result<Self::Apply, Self::Error> {
        #[expect(
            clippy::match_wildcard_for_single_variants,
            reason = "will be dealt with later"
        )]
        match plan {
            RoutePlan::Create(msg) => ctx
                .route()
                .add(msg.clone())
                .execute()
                .await
                .map_err(|e| format!("unable to create route: {e}")),
            RoutePlan::Noop => Ok(()),
            _ => todo!(),
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
