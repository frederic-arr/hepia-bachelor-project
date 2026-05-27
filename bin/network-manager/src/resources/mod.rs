mod link_config;
mod link_spec;

use cos_api_shared::proto::v1;
use cos_api_shared::{
    Reconcilable,
    ReconcilableDriver,
    Resource,
    Specification,
};
pub use link_config::*;
pub use link_spec::*;
use rtnetlink::Handle;

pub enum NetworkResources {
    LinkConfig(Resource<LinkConfigSpec>),
}

impl TryFrom<Option<v1::MetaResource>> for NetworkResources {
    type Error = String;

    fn try_from(value: Option<v1::MetaResource>) -> Result<Self, Self::Error> {
        let value = value.ok_or_else(|| "no resource given")?;
        let schema =
            value.schema().ok_or_else(|| "unable to determine schema")?;

        match schema {
            LinkConfigSpec::SCHEMA => Ok(Self::LinkConfig(value.try_into()?)),
            v => Err(format!(
                "unable to determine schema: no matching schema for {v}"
            )),
        }
    }
}

impl NetworkResources {
    pub async fn reconcile(&self, mut rtnl: Handle) {
        dispatch!(self, resource => {
                let state = resource.refresh(&mut rtnl).await.unwrap();
                let plan = resource.plan(&mut rtnl, state.as_ref()).unwrap();
                resource.apply(&mut rtnl, plan).await.unwrap();
        })
    }
}

pub macro dispatch($self:expr, $variant:ident => $expr:expr) {
    match $self {
        NetworkResources::LinkConfig($variant) => $expr,
        NetworkResources::LinkConfig($variant) => $expr,
    }
}
