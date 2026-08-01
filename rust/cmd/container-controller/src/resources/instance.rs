use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use anyhow::{Context as _, Result, anyhow};
use bollard::Docker;
use bollard::plugin::{
    ContainerCreateBody,
    ContainerSummaryStateEnum,
    EndpointSettings,
    HostConfig,
    NetworkingConfig,
    PortBinding,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder,
    ListContainersOptionsBuilder,
};
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
pub struct InstanceReconciler;

pub type InstanceResource =
    Resource<InstanceSpec, InstanceDerivedSpec, InstanceState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceSpec {
    pub image: String,
    pub runtime: String,
    pub cmd: Option<Vec<String>>,
    pub running: Option<bool>,
    pub ports: Option<Vec<InstancePortSpec>>,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub networks: Option<Vec<String>>,
    pub domainname: Option<String>,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub working_dir: Option<String>,
    pub depends_on: Option<HashSet<Key>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstancePortSpec {
    container_port: u16,
    host_port: Option<u16>,
    host_ip: Option<IpAddr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
pub struct InstanceDerivedSpec {
    name: String,
}

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
        key: Key,
        spec: InstanceSpec,
        resource: Option<InstanceResource>,
    ) -> Result<ValidateResponse<InstanceDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(ValidateResponse {
            derived_spec: InstanceDerivedSpec { name },
            children: vec![],
            dependencies: Self::get_deps(&spec),
        })
    }

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
            InstancePlan::Noop if matches!(resource.phase, Phase::Deleting) => {
                Status::Deleted
            }
            InstancePlan::Noop
                if matches!(resource.phase, Phase::PendingDeletion) =>
            {
                Status::NotReady
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
            vec![resource.derived_spec.name.clone()],
        );

        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();

        let Some(container) = ctx
            .list_containers(Some(options))
            .await
            .context("failed to list containers")?
            .first()
            .cloned()
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
        let plan = match (&resource.phase, cx) {
            (Phase::Running, None) => InstancePlan::Create,
            (Phase::Deleting, Some(_)) => InstancePlan::Delete,
            (Phase::Running, Some(refreshed_state)) => {
                if resource.spec.image == refreshed_state.image {
                    match (
                        resource.spec.running.unwrap_or(true),
                        refreshed_state.running,
                    ) {
                        (true, false) => InstancePlan::Start(
                            resource.derived_spec.name.clone(),
                        ),
                        (false, true) => InstancePlan::Stop(
                            resource.derived_spec.name.clone(),
                        ),
                        (true, true) | (false, false) => InstancePlan::Noop,
                    }
                } else {
                    InstancePlan::Delete
                }
            }
            (Phase::Shutdown, Some(_))
            | (
                Phase::Shutdown | Phase::Deleting | Phase::PendingDeletion,
                None | Some(_),
            ) => InstancePlan::Noop,
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
                    .name(&resource.derived_spec.name)
                    .build();

                let cfg = ContainerCreateBody {
                    image: Some(resource.spec.image.clone()),
                    cmd: resource.spec.cmd.clone(),
                    entrypoint: resource.spec.entrypoint.clone(),
                    env: resource.spec.env.clone(),
                    volumes: resource.spec.volumes.clone(),
                    domainname: resource.spec.domainname.clone(),
                    hostname: resource.spec.hostname.clone(),
                    user: resource.spec.user.clone(),
                    working_dir: resource.spec.working_dir.clone(),
                    host_config: Some(HostConfig {
                        port_bindings: resource.spec.ports.clone().map(
                            |ports| {
                                let bindings = ports.iter().map(|port| {
                                    (
                                        port.container_port.to_string(),
                                        Some(vec![PortBinding {
                                            host_ip: port
                                                .host_ip
                                                .map(|v| v.to_string()),
                                            host_port: port
                                                .host_port
                                                .map(|v| v.to_string()),
                                        }]),
                                    )
                                });

                                bindings.collect()
                            },
                        ),
                        ..Default::default()
                    }),
                    networking_config: resource.spec.networks.clone().map(
                        |nets| NetworkingConfig {
                            endpoints_config: Some(HashMap::from_iter(
                                nets.into_iter().map(|net| {
                                    (net, EndpointSettings::default())
                                }),
                            )),
                        },
                    ),
                    ..Default::default()
                };
                ctx.create_container(Some(opts), cfg).await?;
                if resource.spec.running.unwrap_or(true) {
                    ctx.start_container(&resource.derived_spec.name, None)
                        .await?;
                }

                Ok(())
            }
            InstancePlan::Delete => {
                if let Some(refreshed_state) = cx
                    && refreshed_state.running
                {
                    ctx.stop_container(&resource.derived_spec.name, None)
                        .await?;
                }

                ctx.remove_container(&resource.derived_spec.name, None)
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
        let networks = spec
            .networks
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|net| {
                Identity::Private(PrivateIdentity::Dynamic(Key {
                    schema: "container:network".to_owned(),
                    name: Some(net),
                }))
            });

        let volumes =
            spec.volumes
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|net| {
                    Identity::Private(PrivateIdentity::Dynamic(Key {
                        schema: "container:volume".to_owned(),
                        name: Some(net),
                    }))
                });

        let additional = spec
            .depends_on
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|v| Identity::Private(PrivateIdentity::Dynamic(v)));

        let mut deps = HashSet::from([
            Identity::Private(PrivateIdentity::Dynamic(Key {
                schema: "container:runtime".to_owned(),
                name: Some(spec.runtime.clone()),
            })),
            Identity::Shared(Key {
                schema: "container:image".to_owned(),
                name: Some(format!("{}#{}", spec.runtime, spec.image)),
            }),
        ]);

        deps.extend(networks);
        deps.extend(volumes);
        deps.extend(additional);

        deps
    }
}
