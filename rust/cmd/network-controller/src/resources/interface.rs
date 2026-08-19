use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::{Result, anyhow, bail};
use cos_proto_reconciler::{
    Identity,
    Key,
    PrivateIdentity,
    Resource,
    ResourceResponse,
    Status,
    SubResourceCreate,
    ValidateResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AddressSpec, LinkSpec, LinkSpecType, LinkSpecUnspec, RouteSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceReconciler;

pub type InterfaceResource =
    Resource<InterfaceSpec, InterfaceDerivedSpec, InterfaceState>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSpec {
    pub address: String,
    pub gateway: IpAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDerivedSpec {
    pub name: String,
    pub address: IpAddr,
    pub prefix_len: u8,
}

type InterfaceState = ();

impl InterfaceReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for InterfaceReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceReconciler {
    pub async fn validate(
        &self,
        key: Key,
        spec: InterfaceSpec,
        resource: Option<InterfaceResource>,
    ) -> Result<ValidateResponse<InterfaceDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;
        let Some((addr, prefix)) = spec.address.split_once('/') else {
            bail!("invalid address format");
        };

        let address = IpAddr::parse_ascii(addr.as_bytes())?;
        let prefix_len = prefix.parse::<u8>()?;

        let dspec = InterfaceDerivedSpec {
            name,
            address,
            prefix_len,
        };

        Ok(ValidateResponse {
            children: Self::get_children(&spec, &dspec)?,
            derived_spec: dspec,
            dependencies: HashSet::new(),
        })
    }

    pub async fn reconcile(
        &self,
        resource: InterfaceResource,
    ) -> Result<ResourceResponse<Option<InterfaceState>>> {
        let children =
            Self::get_children(&resource.spec, &resource.derived_spec)?;
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: None,
                children: vec![],
                dependencies: HashSet::new(),
            });
        }

        Ok(ResourceResponse {
            status: Status::Done,
            state: None,
            children,
            dependencies: HashSet::new(),
        })
    }

    async fn validate_new_spec(&self, _spec: &InterfaceSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &InterfaceResource,
        spec: &InterfaceSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_children(
        spec: &InterfaceSpec,
        dspec: &InterfaceDerivedSpec,
    ) -> Result<Vec<SubResourceCreate<Value>>> {
        Ok(vec![
            SubResourceCreate::<Value> {
                id: Identity::Private(PrivateIdentity::Dynamic(Key {
                    schema: "network:link".to_owned(),
                    name: Some(dspec.name.clone()),
                })),
                spec: serde_json::to_value(LinkSpec {
                    admin_up: true,
                    link_type: LinkSpecType::Unspec(LinkSpecUnspec {}),
                })?,
            },
            SubResourceCreate::<Value> {
                id: Identity::Private(PrivateIdentity::Dynamic(Key {
                    schema: "network:address".to_owned(),
                    name: Some(format!("dyn-{}-iface", dspec.name)),
                })),
                spec: serde_json::to_value(AddressSpec {
                    dev: dspec.name.clone(),
                    address: dspec.address,
                    prefix_len: dspec.prefix_len,
                })?,
            },
            SubResourceCreate::<Value> {
                id: Identity::Private(PrivateIdentity::Dynamic(Key {
                    schema: "network:route".to_owned(),
                    name: Some(format!("dyn-{}-iface", dspec.name)),
                })),
                spec: serde_json::to_value(match spec.gateway {
                    IpAddr::V4(gw) => RouteSpec::Ipv4 {
                        destination: Ipv4Addr::UNSPECIFIED,
                        prefix_len: 0,
                        gateway: gw,
                        parent: Some(dspec.name.clone()),
                    },
                    IpAddr::V6(gw) => RouteSpec::Ipv6 {
                        destination: Ipv6Addr::UNSPECIFIED,
                        prefix_len: 0,
                        gateway: gw,
                        parent: Some(dspec.name.clone()),
                    },
                })?,
            },
        ])
    }
}
