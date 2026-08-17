use std::collections::HashSet;

use anyhow::{Result, anyhow};
use bollard::Docker;
use bollard::plugin::VolumeCreateRequest;
use bollard::query_parameters::RemoveVolumeOptions;
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
pub struct VolumeReconciler;

pub type VolumeResource = Resource<VolumeSpec, VolumeDerivedSpec, VolumeState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
pub struct VolumeDerivedSpec {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct VolumeState {
    pub mountpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumePlan {
    Create,
    Delete,
    Noop,
}

impl VolumeReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for VolumeReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl VolumeReconciler {
    pub async fn validate(
        &self,
        key: Key,
        spec: VolumeSpec,
        resource: Option<VolumeResource>,
    ) -> Result<ValidateResponse<VolumeDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(ValidateResponse {
            derived_spec: VolumeDerivedSpec { name },
            children: vec![],
            dependencies: Self::get_deps(&spec),
        })
    }

    pub async fn reconcile(
        &self,
        resource: VolumeResource,
    ) -> Result<ResourceResponse<VolumeState>> {
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
            VolumePlan::Noop if matches!(resource.phase, Phase::Deleting) => {
                Status::Deleted
            }
            VolumePlan::Noop
                if matches!(resource.phase, Phase::PendingDeletion) =>
            {
                Status::NotReady
            }
            VolumePlan::Noop => Status::Ready,
            VolumePlan::Create | VolumePlan::Delete => Status::NotReady,
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
        resource: &VolumeResource,
        ctx: &Docker,
    ) -> Result<Option<VolumeState>> {
        let Ok(vol) = ctx.inspect_volume(&resource.derived_spec.name).await
        else {
            return Ok(None);
        };

        Ok(Some(VolumeState {
            mountpoint: vol.mountpoint,
        }))
    }

    async fn plan(
        &self,
        resource: &VolumeResource,
        cx: Option<&VolumeState>,
    ) -> Result<VolumePlan> {
        let plan = match (&resource.phase, cx) {
            (Phase::Running, None) => VolumePlan::Create,
            (Phase::Deleting, Some(_)) => VolumePlan::Delete,
            (Phase::Running | Phase::Shutdown, Some(_))
            | (
                Phase::Shutdown | Phase::Deleting | Phase::PendingDeletion,
                None | Some(_),
            ) => VolumePlan::Noop,
        };

        Ok(plan)
    }

    async fn apply(
        &self,
        resource: &VolumeResource,
        plan: &VolumePlan,
        _cx: Option<&VolumeState>,
        ctx: &Docker,
    ) -> Result<()> {
        match plan {
            VolumePlan::Create => {
                ctx.create_volume(VolumeCreateRequest {
                    name: Some(resource.derived_spec.name.clone()),
                    ..Default::default()
                })
                .await?;

                Ok(())
            }
            VolumePlan::Delete => ctx
                .remove_volume(
                    &resource.derived_spec.name,
                    None::<RemoveVolumeOptions>,
                )
                .await
                .map_err(Into::into),
            VolumePlan::Noop => Ok(()),
        }
    }

    async fn validate_new_spec(&self, _spec: &VolumeSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &VolumeResource,
        spec: &VolumeSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_deps(spec: &VolumeSpec) -> HashSet<Identity> {
        HashSet::from([Identity::Private(PrivateIdentity::Dynamic(Key {
            schema: "container:runtime".to_owned(),
            name: Some(spec.runtime.clone()),
        }))])
    }
}
