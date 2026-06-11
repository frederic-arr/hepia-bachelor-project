use std::collections::HashMap;

use bollard::Docker;
use bollard::plugin::{ContainerCreateBody, ContainerSummaryStateEnum};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder,
    CreateImageOptions,
    CreateImageOptionsBuilder,
    ListContainersOptionsBuilder,
    ListImagesOptionsBuilder,
};
use cos_api_reconciler::ReconcileDynamicResourceRequest;
use cos_api_reconciler::proto::v1;
use cos_api_reconciler_server::Reconcilable;
use derive_builder::Builder;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

pub struct Container;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContainerSpec {
    pub image: String,
    pub running: bool,
    pub cmd: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct ContainerState {
    pub id: String,
    pub image: String,
    pub running: bool,
    pub cmd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerPlan {
    Create,
    Delete,
    Pull(String),
    Start(String),
    Stop(String),
    Noop,
}

impl Reconcilable for Container {
    type Apply = ();
    type Context = Docker;
    type Input = ReconcileDynamicResourceRequest<ContainerSpec, ContainerState>;
    type Output = v1::ReconcileDynamicResourceResponse;
    type Plan = ContainerPlan;
    type State = Option<ContainerState>;

    const SCHEMA: &'static str = "res#containeros::container::container";

    async fn refresh(
        ctx: &mut Self::Context,
        request: &Self::Input,
    ) -> Self::State {
        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![request.name.clone()]);

        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();

        let container = ctx
            .list_containers(Some(options))
            .await
            .unwrap()
            .first()
            .cloned()?;

        let mut state = ContainerStateBuilder::default();
        if let Some(id) = container.id {
            state.id(id);
        }

        if let Some(image) = container.image {
            state.image(image);
        }

        if let Some(cmd) = container.command {
            state.cmd(cmd);
        }

        match container.state.unwrap() {
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

        state.build().map(Some).unwrap()
    }

    async fn plan(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
    ) -> Self::Plan {
        let mut filters = HashMap::new();
        filters.insert("reference", vec![request.spec.image.as_str()]);

        let opts = ListImagesOptionsBuilder::default()
            .filters(&filters)
            .build();

        let Some(images) = ctx.list_images(Some(opts)).await.unwrap().first()
        else {
            return ContainerPlan::Pull(request.spec.image.clone());
        };

        let Some(refreshed_state) = refreshed_state else {
            return ContainerPlan::Create;
        };

        if request.spec.image != refreshed_state.image {
            return ContainerPlan::Delete;
        }

        if request.spec.cmd.join(" ") != refreshed_state.cmd {
            return ContainerPlan::Delete;
        }

        match (request.spec.running, refreshed_state.running) {
            (true, false) => ContainerPlan::Start(request.name.clone()),
            (false, true) => ContainerPlan::Stop(request.name.clone()),
            (true, true) | (false, false) => ContainerPlan::Noop,
        }
    }

    async fn apply(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Self::Apply {
        match plan {
            ContainerPlan::Create => {
                let opts = CreateContainerOptionsBuilder::default()
                    .name(&request.name)
                    .build();

                let cfg = ContainerCreateBody {
                    image: Some(request.spec.image.clone()),
                    cmd: Some(request.spec.cmd.clone()),
                    ..Default::default()
                };
                ctx.create_container(Some(opts), cfg).await.unwrap();
                if request.spec.running {
                    ctx.start_container(&request.name, None).await;
                }
            }
            ContainerPlan::Delete => {
                if let Some(refreshed_state) = refreshed_state
                    && refreshed_state.running
                {
                    ctx.stop_container(&request.name, None).await;
                }

                ctx.remove_container(&request.name, None).await;
            }
            ContainerPlan::Pull(image) => {
                let opts = CreateImageOptionsBuilder::default()
                    .from_image(image)
                    .build();

                let mut stream = ctx.create_image(Some(opts), None, None);
                tokio::spawn(async move {
                    while let Some(r) = stream.next().await {
                        dbg!(r);
                    }
                });
            }
            ContainerPlan::Start(name) => {
                ctx.start_container(name, None).await.unwrap();
            }
            ContainerPlan::Stop(name) => {
                ctx.stop_container(name, None).await.unwrap();
            }
            ContainerPlan::Noop => {}
        }
    }

    fn update(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Self::Output> {
        let response = v1::ReconcileDynamicResourceResponse {
            children: vec![],
            state: rmp_serde::to_vec_named(refreshed_state).unwrap(),
        };

        std::future::ready(response)
    }
}
