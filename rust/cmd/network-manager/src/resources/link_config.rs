use std::collections::{HashMap, HashSet};
use std::ops::Not;

use cos_api_reconciler::proto::v1;
use cos_api_reconciler::{
    Identity,
    ReconcileUserConfigRequest,
    SubResourceCreate,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::resources::{LinkSpec, Reconcilable};

pub struct LinkConfig;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkConfigSpec {
    pub up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LinkConfigState {}

#[derive(Debug)]
pub struct LinkConfigPlan {
    children: Vec<SubResourceCreate>,
}

impl Reconcilable for LinkConfig {
    type Apply = ();
    type Input = ReconcileUserConfigRequest<LinkConfigSpec, LinkConfigState>;
    type Output = v1::ReconcileUserConfigResponse;
    type Plan = LinkConfigPlan;
    type State = LinkConfigState;

    const SCHEMA: &'static str = "config#containeros::net::link";

    fn refresh(request: &Self::Input) -> impl Future<Output = Self::State> {
        std::future::ready(LinkConfigState {})
    }

    fn plan(
        request: &Self::Input,
        refreshed_state: &Self::State,
    ) -> impl Future<Output = Self::Plan> {
        let link_id = Identity {
            schema: "res#containeros::net::link".to_string(),
            name: request.name.clone(),
        };

        let link_spec = SubResourceCreate {
            schema: link_id.schema.clone(),
            name: link_id.name,
            spec: rmp_serde::to_vec(&LinkSpec {
                admin_up: request.spec.up,
            })
            .unwrap(),
        };

        let children = vec![link_spec];
        let plan = LinkConfigPlan { children };
        std::future::ready(plan)
    }

    async fn apply(
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Self::Apply {
    }

    fn update(
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> impl Future<Output = Self::Output> {
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

        std::future::ready(response)
    }
}
