use std::collections::HashSet;

use anyhow::{Result, anyhow};
use bollard::Docker;
use bollard::plugin::NetworkCreateRequest;
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
use serde::{Deserialize, Serialize};

use crate::RuntimeDerivedSpec;

#[derive(Debug, Clone)]
pub struct NetworkReconciler;

pub type NetworkResource =
    Resource<NetworkSpec, NetworkDerivedSpec, NetworkState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkSpec {
    pub runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
pub struct NetworkDerivedSpec {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct NetworkState {
    pub id: String,
    pub containers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPlan {
    Create,
    Delete,
    Noop,
}

impl NetworkReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NetworkReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkReconciler {
    pub async fn validate(
        &self,
        key: Key,
        spec: NetworkSpec,
        resource: Option<NetworkResource>,
    ) -> Result<ValidateResponse<NetworkDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(ValidateResponse {
            derived_spec: NetworkDerivedSpec { name },
            children: vec![],
            dependencies: Self::get_deps(&spec),
        })
    }

    pub async fn reconcile(
        &self,
        resource: NetworkResource,
    ) -> Result<ResourceResponse<NetworkState>> {
        let Some(rt) = resource
            .dependencies
            .iter()
            .find(|v| v.id.schema() == "container:runtime")
        else {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![],
                dependencies: Self::get_deps(&resource.spec),
            });
        };

        let rt: RuntimeDerivedSpec =
            serde_json::from_value(rt.derived_spec.clone())?;

        let client =
            Docker::connect_with_host(&format!("tcp://127.0.0.1:{}", rt.port))?;

        let cx = match self.refresh(&resource, &client).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("refresh: {err:#}").into()),
                    state: resource.state,
                    children: vec![],
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        };

        let state = &cx;

        let plan = match self.plan(&resource, cx.as_ref()).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("plan: {err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        };

        let () =
            match self.apply(&resource, &plan, state.as_ref(), &client).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(ResourceResponse {
                        status: Status::Error(format!("apply: {err:#}").into()),
                        state: state.clone(),
                        children: vec![],
                        dependencies: Self::get_deps(&resource.spec),
                    });
                }
            };

        let new_cx = match self.refresh(&resource, &client).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("rerefresh: {err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        };

        let state = &new_cx;
        let new_plan = match self.plan(&resource, state.as_ref()).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("replan: {err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        };

        let status = match new_plan {
            NetworkPlan::Noop if matches!(resource.phase, Phase::Deleting) => {
                Status::Deleted
            }
            NetworkPlan::Noop
                if matches!(resource.phase, Phase::PendingDeletion) =>
            {
                Status::NotReady
            }
            NetworkPlan::Noop => Status::Ready,
            NetworkPlan::Create | NetworkPlan::Delete => Status::NotReady,
        };

        Ok(ResourceResponse {
            status,
            state: state.clone(),
            children: vec![],
            dependencies: Self::get_deps(&resource.spec),
        })
    }

    async fn refresh(
        &self,
        resource: &NetworkResource,
        ctx: &Docker,
    ) -> Result<Option<NetworkState>> {
        let Ok(net) =
            ctx.inspect_network(&resource.derived_spec.name, None).await
        else {
            return Ok(None);
        };

        let containers = net
            .containers
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| v.name.unwrap_or(k))
            .collect();

        Ok(Some(NetworkState {
            id: net.id.ok_or_else(|| anyhow!("no network id"))?,
            containers,
        }))
    }

    async fn plan(
        &self,
        resource: &NetworkResource,
        cx: Option<&NetworkState>,
    ) -> Result<NetworkPlan> {
        let plan = match (&resource.phase, cx) {
            (Phase::Running, None) => NetworkPlan::Create,
            (Phase::Deleting, Some(_)) => NetworkPlan::Delete,
            (Phase::Running | Phase::Shutdown, Some(_))
            | (
                Phase::Shutdown | Phase::Deleting | Phase::PendingDeletion,
                None | Some(_),
            ) => NetworkPlan::Noop,
        };

        Ok(plan)
    }

    async fn apply(
        &self,
        resource: &NetworkResource,
        plan: &NetworkPlan,
        _cx: Option<&NetworkState>,
        ctx: &Docker,
    ) -> Result<()> {
        match plan {
            NetworkPlan::Create => {
                ctx.create_network(NetworkCreateRequest {
                    name: resource.derived_spec.name.clone(),
                    ..Default::default()
                })
                .await?;

                Ok(())
            }
            NetworkPlan::Delete => ctx
                .remove_network(&resource.derived_spec.name)
                .await
                .map_err(Into::into),
            NetworkPlan::Noop => Ok(()),
        }
    }

    async fn validate_new_spec(&self, _spec: &NetworkSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &NetworkResource,
        spec: &NetworkSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_deps(spec: &NetworkSpec) -> HashSet<Identity> {
        HashSet::from([Identity::Private(PrivateIdentity::Dynamic(Key {
            schema: "container:runtime".to_owned(),
            name: Some(spec.runtime.clone()),
        }))])
    }
}
