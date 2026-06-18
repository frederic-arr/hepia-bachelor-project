use std::net::{IpAddr, Ipv4Addr};

use cos_api_reconciler::ReconcileDynamicResourceRequest;
use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::Reconcilable;
use derive_builder::Builder;
use futures::{StreamExt, TryStreamExt};
use rtnetlink::packet_route::address::{AddressAttribute, AddressMessage};
use rtnetlink::packet_route::link::{LinkAttribute, LinkFlags, LinkMessage};
use rtnetlink::{
    AddressAddRequest,
    AddressMessageBuilder,
    Handle,
    LinkDummy,
    LinkMessageBuilder,
    LinkUnspec,
    new_connection,
};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

pub struct Address;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AddressSpec {
    pub link_name: String,
    pub address: Ipv4Addr,
    pub prefix_len: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct AddressState {
    pub index: u32,
    pub address: Ipv4Addr,
    pub prefix_len: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressPlan {
    Create {
        link_index: u32,
        address: IpAddr,
        prefix_len: u8,
    },
    Recreate {
        index: u32,
        msg: AddressMessage,
    },
    Delete(u32),
    Noop,
}

impl Reconcilable for Address {
    type Apply = ();
    type Context = Handle;
    type Error = String;
    type Input = ReconcileDynamicResourceRequest<AddressSpec, AddressState>;
    type Output = v1::ReconcileDynamicResourceResponse;
    type Plan = AddressPlan;
    type State = (u32, Option<AddressState>);

    const SCHEMA: &'static str = "res#containeros::net::address";

    async fn refresh(
        ctx: &mut Self::Context,
        input: &Self::Input,
    ) -> Result<Self::State, Self::Error> {
        let mut state = AddressStateBuilder::default();
        let mut links = ctx
            .link()
            .get()
            .match_name(input.spec.link_name.clone())
            .execute();

        let link = links.try_next().await.unwrap().unwrap();
        let link_index = link.header.index;

        let mut addresses = ctx
            .address()
            .get()
            .set_address_filter(input.spec.address.into())
            .set_prefix_length_filter(input.spec.prefix_len)
            .set_link_index_filter(link_index)
            .execute();

        let address = addresses
            .try_next()
            .await
            .expect("at least one RTNL message");
        assert!(
            addresses.next().await.is_none(),
            "got multiple links while only one was expected"
        );

        let Some(address) = address else {
            return Ok((link_index, None));
        };

        state.index(address.header.index);
        state.prefix_len(address.header.prefix_len);
        for nla in address.attributes {
            use rtnetlink::packet_route::link;
            if let AddressAttribute::Address(IpAddr::V4(addr)) = nla {
                state.address(addr);
            }
        }

        Ok((link_index, state.build().map(Some).unwrap()))
    }

    fn plan(
        ctx: &mut Self::Context,
        input: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>> {
        let plan = match (&input.state, refreshed_state) {
            (None, (link_index, None)) => AddressPlan::Create {
                link_index: *link_index,
                address: input.spec.address.into(),
                prefix_len: input.spec.prefix_len,
            },
            _ => AddressPlan::Noop,
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
            AddressPlan::Create {
                link_index,
                address,
                prefix_len,
            } => ctx
                .address()
                .add(*link_index, *address, *prefix_len)
                .execute()
                .await
                .map_err(|e| format!("unable to create link: {e}")),
            AddressPlan::Noop => Ok(()),
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
        let new_state = Self::refresh(ctx, input).await?.1.unwrap();
        Ok(v1::ReconcileDynamicResourceResponse {
            state: rmp_serde::to_vec_named(&new_state).unwrap(),
            children: vec![],
        })
    }
}
