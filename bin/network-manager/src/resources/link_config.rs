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
pub enum LinkConfigPlan {
    Noop,
    Op {
        created_children: Vec<SubResourceCreate>,
        removed_children: Vec<Identity>,
    },
}

impl Reconcilable for LinkConfig {
    type Apply = ();
    type Input = ReconcileUserConfigRequest<LinkConfigSpec, LinkConfigState>;
    type Output = v1::ReconcileUserConfigResponse;
    type Plan = LinkConfigPlan;
    type State = LinkConfigState;

    const SCHEMA: &'static str = "config#containeros::net::link";

    async fn refresh(request: &Self::Input) -> Self::State {
        LinkConfigState {}
    }

    async fn plan(
        request: &Self::Input,
        refreshed_state: &Self::State,
    ) -> Self::Plan {
        let link_id = Identity {
            schema: "res#containeros::net::link".to_string(),
            name: request.name.clone(),
        };

        let link_spec = SubResourceCreate {
            schema: link_id.schema.clone(),
            name: link_id.name.clone(),
            spec: rmp_serde::to_vec(&LinkSpec {
                admin_up: request.spec.up,
            })
            .unwrap(),
        };

        let required_children = HashMap::from([(link_id, link_spec)]);

        let created_children = required_children
            .iter()
            .filter_map(|(id, spec)| {
                request.children.contains(id).not().then_some(spec)
            })
            .cloned()
            .collect::<Vec<_>>();

        let removed_children = request
            .children
            .iter()
            .filter(|child| !required_children.contains_key(child))
            .cloned()
            .collect::<Vec<_>>();

        if removed_children.is_empty() && created_children.is_empty() {
            return LinkConfigPlan::Noop;
        }

        LinkConfigPlan::Op {
            created_children,
            removed_children,
        }
    }

    async fn apply(
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
    ) -> Self::Apply {
        ();
    }

    async fn update(
        request: &Self::Input,
        refreshed_state: &Self::State,
        plan: &Self::Plan,
        apply: &Self::Apply,
    ) -> Self::Output {
        match plan {
            LinkConfigPlan::Noop => v1::ReconcileUserConfigResponse {
                create: vec![],
                state: rmp_serde::to_vec_named(refreshed_state).unwrap(),
            },
            LinkConfigPlan::Op {
                created_children,
                removed_children,
            } => v1::ReconcileUserConfigResponse {
                create: created_children
                    .iter()
                    .map(|c| v1::DynamicResourceCreate {
                        schema: c.schema.clone(),
                        name: c.name.clone(),
                        spec: c.spec.clone(),
                    })
                    .collect(),
                state: rmp_serde::to_vec_named(refreshed_state).unwrap(),
            },
        }
    }
}
