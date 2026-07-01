use std::collections::{HashMap, HashSet};
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

use crate::resources::ContainerSpec;

pub struct ContainerConfig;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContainerConfigSpec {
    pub image: String,
    pub running: bool,
    pub cmd: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContainerConfigState {}

#[derive(Debug)]
pub struct ContainerConfigPlan {
    children: Vec<SubResourceCreate>,
}

impl Reconcilable for ContainerConfig {
    type Apply = ();
    type Context = ();
    type Error = !;
    type Input =
        ReconcileUserConfigRequest<ContainerConfigSpec, ContainerConfigState>;
    type Output = v1::ReconcileUserConfigResponse;
    type Plan = ContainerConfigPlan;
    type State = ContainerConfigState;

    const SCHEMA: &'static str = "config#containeros::container::container";

    fn refresh(
        ctx: &mut Self::Context,
        request: &Self::Input,
    ) -> impl Future<Output = Result<Self::State, Self::Error>> {
        std::future::ready(Ok(ContainerConfigState {}))
    }

    fn plan(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Result<Self::Plan, Self::Error>> {
        let container_resource_id = Identity {
            schema: "res#containeros::container::container".to_string(),
            name: request.name.clone(),
        };

        let container_spec = SubResourceCreate {
            schema: container_resource_id.schema.clone(),
            name: container_resource_id.name,
            spec: rmp_serde::to_vec(&ContainerSpec {
                image: request.spec.image.clone(),
                running: request.spec.running,
                cmd: request.spec.cmd.clone(),
            })
            .unwrap(),
        };

        let children = vec![container_spec];
        let plan = ContainerConfigPlan { children };
        std::future::ready(Ok(plan))
    }

    fn apply(
        ctx: &mut Self::Context,
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> impl Future<Output = Result<Self::Apply, Self::Error>> {
        std::future::ready(Ok(()))
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
