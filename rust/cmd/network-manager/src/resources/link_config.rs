use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::ops::Not;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    Identity,
    ReconcileUserConfigRequest,
    SubResourceCreate,
};
use cos_api_reconciler_server::Reconcilable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::resources::{AddressSpec, LinkSpec, LinkType, RouteSpec};

pub struct LinkConfig;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkConfigSpec {
    pub up: bool,

    pub mtu: Option<u32>,
    pub address: Option<[u8; 6]>,
    pub broadcast: Option<[u8; 6]>,
    pub altnames: Option<Vec<String>>,
    pub arp: Option<bool>,
    pub promiscuous: Option<bool>,
    pub link_type: LinkConfigType,

    pub ip_address: Ipv4Addr,
    pub ip_subnet: u8,
    pub ip_gateway: Ipv4Addr,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LinkConfigType {
    Dummy,
    Ethernet,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkConfigState {}

#[derive(Debug)]
pub struct LinkConfigPlan {
    children: Vec<SubResourceCreate>,
}

impl Reconcilable for LinkConfig {
    type Apply = ();
    type Context = ();
    type Error = !;
    type Input = ReconcileUserConfigRequest<LinkConfigSpec, LinkConfigState>;
    type Output = v1::ReconcileUserConfigResponse;
    type Plan = LinkConfigPlan;
    type State = LinkConfigState;

    const SCHEMA: &'static str = "config#containeros::net::link";

    fn refresh(
        ctx: &mut Self::Context,
        request: &Self::Input,
    ) -> impl Future<Output = Result<Self::State, Self::Error>> {
        std::future::ready(Ok(LinkConfigState {}))
    }

    fn plan(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>> {
        let link_id = Identity {
            schema: "res#containeros::net::link".to_string(),
            name: request.name.clone(),
        };

        let addr_id = Identity {
            schema: "res#containeros::net::address".to_string(),
            name: request.name.clone(),
        };

        let route_id = Identity {
            schema: "res#containeros::net::route".to_string(),
            name: request.name.clone(),
        };

        let link_spec = SubResourceCreate {
            schema: link_id.schema.clone(),
            name: link_id.name,
            spec: rmp_serde::to_vec(&LinkSpec {
                admin_up: request.spec.up,
                mtu: request.spec.mtu,
                address: request.spec.address,
                broadcast: request.spec.broadcast,
                altnames: request
                    .spec
                    .altnames
                    .clone()
                    .unwrap_or_else(|| vec![]),
                arp: request.spec.arp.unwrap_or(true),
                promiscuous: request.spec.promiscuous.unwrap_or(false),
                link_type: match request.spec.link_type {
                    LinkConfigType::Dummy { .. } => LinkType::Dummy,
                    LinkConfigType::Ethernet { .. } => LinkType::Ethernet,
                },
            })
            .unwrap(),
        };

        let addr_spec = SubResourceCreate {
            schema: addr_id.schema.clone(),
            name: addr_id.name,
            spec: rmp_serde::to_vec(&AddressSpec {
                link_name: request.name.clone(),
                address: request.spec.ip_address,
                prefix_len: request.spec.ip_subnet,
            })
            .unwrap(),
        };

        let route_spec = SubResourceCreate {
            schema: route_id.schema.clone(),
            name: route_id.name,
            spec: rmp_serde::to_vec(&RouteSpec {
                gateway: request.spec.ip_gateway,
                destination: Ipv4Addr::new(0, 0, 0, 0),
                prefix_len: 0,
            })
            .unwrap(),
        };

        let children = vec![link_spec, addr_spec, route_spec];
        let plan = LinkConfigPlan { children };
        std::future::ready(Ok(plan))
    }

    async fn apply(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Result<Self::Apply, Self::Error> {
        Ok(())
    }

    fn update(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> {
        let response = v1::ReconcileUserConfigResponse {
            children: plan
                .children
                .iter()
                .map(|c| v1::SubResourceWrite {
                    schema: c.schema.clone(),
                    name: c.name.clone(),
                    spec: c.spec.clone(),
                })
                .collect(),
            state: rmp_serde::to_vec_named(refreshed_state).unwrap(),
        };

        std::future::ready(Ok(response))
    }
}
