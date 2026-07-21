use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use bollard::Docker;
use bollard::plugin::{ContainerCreateBody, ContainerSummaryStateEnum};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder,
    ListContainersOptionsBuilder,
};
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
use serde::{Deserialize, Serialize};

use crate::RuntimeDerivedSpec;

#[derive(Debug, Clone)]
pub struct InstanceReconciler;

pub type InstanceResource =
    Resource<InstanceSpec, InstanceDerivedSpec, InstanceState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceSpec {
    pub name: String,
    pub image: String,
    pub runtime: String,
    pub running: bool,
    pub cmd: Vec<String>,
}

pub type InstanceDerivedSpec = ();

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct InstanceState {
    pub id: String,
    pub image: String,
    pub running: bool,
    pub cmd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstancePlan {
    Create,
    Delete,
    Start(String),
    Stop(String),
    Noop,
}

impl InstanceReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for InstanceReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceReconciler {
    pub async fn validate(
        &self,
        spec: InstanceSpec,
        resource: Option<InstanceResource>,
    ) -> Result<ValidateResponse<InstanceDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: (),
            children: vec![],
            dependencies: Self::get_deps(&spec),
        })
    }

    #[expect(clippy::too_many_lines, reason = "TODO")]
    pub async fn reconcile(
        &self,
        resource: InstanceResource,
    ) -> Result<ResourceResponse<InstanceState>> {
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

        let wait_for_pull = resource
            .dependencies
            .iter()
            .find(|v| v.id.schema() == "container:image")
            .is_none_or(|v| v.status != Status::Done);

        if wait_for_pull {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![],
                dependencies: Self::get_deps(&resource.spec),
            });
        }

        let client =
            Docker::connect_with_host(&format!("tcp://127.0.0.1:{}", rt.port))?;

        let cx = match self.refresh(&resource, &client).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
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
                    status: Status::Error(format!("{err:#}").into()),
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
                        status: Status::Error(format!("{err:#}").into()),
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
                    status: Status::Error(format!("{err:#}").into()),
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
                    status: Status::Error(format!("{err:#}").into()),
                    state: state.clone(),
                    children: vec![],
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        };

        let status = match new_plan {
            InstancePlan::Noop if matches!(resource.phase, Phase::Teardown) => {
                Status::Deleted
            }
            InstancePlan::Noop => Status::Ready,
            InstancePlan::Create
            | InstancePlan::Delete
            | InstancePlan::Start(_)
            | InstancePlan::Stop(_) => Status::NotReady,
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
        resource: &InstanceResource,
        ctx: &Docker,
    ) -> Result<Option<InstanceState>> {
        let mut filters = HashMap::new();
        filters.insert(
            "name".to_owned(),
            vec![resource.spec.name.clone()],
        );

        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();

        let Some(container) =
            ctx.list_containers(Some(options)).await?.first().cloned()
        else {
            return Ok(None);
        };

        let mut state = InstanceStateBuilder::default();
        if let Some(id) = container.id {
            state.id(id);
        }

        if let Some(image) = container.image {
            state.image(image);
        }

        if let Some(cmd) = container.command {
            state.cmd(cmd);
        }

        match container
            .state
            .ok_or_else(|| anyhow!("no container state"))?
        {
            ContainerSummaryStateEnum::RUNNING
            | ContainerSummaryStateEnum::RESTARTING => {
                state.running(true);
            }

            ContainerSummaryStateEnum::EMPTY
            | ContainerSummaryStateEnum::CREATED
            | ContainerSummaryStateEnum::PAUSED
            | ContainerSummaryStateEnum::EXITED
            | ContainerSummaryStateEnum::REMOVING
            | ContainerSummaryStateEnum::DEAD
            | ContainerSummaryStateEnum::STOPPING => {
                state.running(false);
            }
        }

        Ok(state.build().map(Some)?)
    }

    async fn plan(
        &self,
        resource: &InstanceResource,
        cx: Option<&InstanceState>,
    ) -> Result<InstancePlan> {
        let Some(refreshed_state) = cx else {
            return Ok(InstancePlan::Create);
        };

        if resource.spec.image != refreshed_state.image {
            return Ok(InstancePlan::Delete);
        }

        if resource.spec.cmd.join(" ") != refreshed_state.cmd {
            return Ok(InstancePlan::Delete);
        }

        let plan = match (resource.spec.running, refreshed_state.running) {
            (true, false) => InstancePlan::Start(resource.spec.name.clone()),
            (false, true) => InstancePlan::Stop(resource.spec.name.clone()),
            (true, true) | (false, false) => InstancePlan::Noop,
        };

        Ok(plan)
    }

    async fn apply(
        &self,
        resource: &InstanceResource,
        plan: &InstancePlan,
        cx: Option<&InstanceState>,
        ctx: &Docker,
    ) -> Result<()> {
        match plan {
            InstancePlan::Create => {
                let opts = CreateContainerOptionsBuilder::default()
                    .name(&resource.spec.name)
                    .build();

                let cfg = ContainerCreateBody {
                    image: Some(resource.spec.image.clone()),
                    cmd: Some(resource.spec.cmd.clone()),
                    ..Default::default()
                };
                ctx.create_container(Some(opts), cfg).await?;
                if resource.spec.running {
                    ctx.start_container(&resource.spec.name, None).await?;
                }

                Ok(())
            }
            InstancePlan::Delete => {
                if let Some(refreshed_state) = cx
                    && refreshed_state.running
                {
                    ctx.stop_container(&resource.spec.name, None).await?;
                }

                ctx.remove_container(&resource.spec.name, None)
                    .await
                    .map_err(Into::into)
            }
            InstancePlan::Start(name) => {
                ctx.start_container(name, None).await.map_err(Into::into)
            }
            InstancePlan::Stop(name) => {
                ctx.stop_container(name, None).await.map_err(Into::into)
            }
            InstancePlan::Noop => Ok(()),
        }
    }

    async fn validate_new_spec(&self, _spec: &InstanceSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &InstanceResource,
        spec: &InstanceSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_deps(spec: &InstanceSpec) -> HashSet<Identity> {
        HashSet::from([
            Identity::Dynamic(Key {
                schema: "container:runtime".to_owned(),
                name: Some(spec.runtime.clone()),
            }),
            Identity::Shared(Key {
                schema: "container:image".to_owned(),
                name: Some(format!("{}#{}", spec.runtime, spec.image)),
            }),
        ])
    }
}
