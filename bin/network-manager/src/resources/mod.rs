mod link_spec;

use cos_api_reconciler_server::{Reconcilable, ReconcilableDriver};
use cos_api_shared::proto::v1;
use cos_api_shared::{DynamicResource, Resource, Specification};
pub use link_spec::*;
use rtnetlink::Handle;

#[derive(Debug, Clone)]
pub enum NetworkError {}

pub enum NetworkResources {
    LinkSpec(<LinkSpec as Reconcilable>::Resource),
}

impl TryFrom<Option<v1::MetaResource>> for NetworkResources {
    type Error = String;

    fn try_from(value: Option<v1::MetaResource>) -> Result<Self, Self::Error> {
        let value = value.ok_or_else(|| "no resource given")?;
        let schema =
            value.schema().ok_or_else(|| "unable to determine schema")?;

        match schema {
            LinkSpec::SCHEMA => Ok(Self::LinkSpec(value.try_into()?)),
            v => Err(format!(
                "unable to determine schema: no matching schema for {v}"
            )),
        }
    }
}
