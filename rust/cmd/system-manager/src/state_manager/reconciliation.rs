use std::collections::hash_map::Entry;
use std::time::{Duration, SystemTime};

use cos_api_reconciler::proto::v1;
use invariant_macros::invariant_violation;
use tokio::time::Instant;
use tokio_stream::StreamExt;

use crate::resources::{
    DynamicResource,
    Identity,
    Resource,
    ResourceState,
    Spec,
    State,
};
use crate::state_manager::StateManager;

impl StateManager {
    pub async fn reconciliation_loop(&mut self) {
        loop {
            let ids = self.resources.keys().cloned().collect::<Vec<_>>();
            for id in ids {
                self.reconciliation_tick(&id).await;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn reconciliation_tick(&mut self, id: &Identity) {
        tracing::info!("reconciliation attempt of {id}");

        let Some(mut client) = self.get_client_for_id(id) else {
            tracing::error!("no clients for {id}");
            return;
        };

        let children = self
            .resources
            .iter()
            .filter_map(|(id, res)| match res {
                Resource::UserConfig(res) => None,
                Resource::DynamicResource(res) => {
                    (&res.owner == id).then_some(v1::SubResourceRead {
                        schema: id.schema.clone(),
                        name: id.name.clone(),
                        spec: res.spec.0.clone(),
                        state: match res.state.clone() {
                            ResourceState::Unset => {
                                Some(v1::sub_resource_read::State::Unset(()))
                            }
                            ResourceState::Set(state) => Some(
                                v1::sub_resource_read::State::Ready(state.0),
                            ),
                        },
                    })
                }
            })
            .collect::<Vec<_>>();

        let Entry::Occupied(mut e) = self.resources.entry(id.clone()) else {
            invariant_violation!(
                "reconciliation scheduled on a non-existing resource: {id}"
            );
        };

        match &mut e.get_mut() {
            Resource::UserConfig(res) => {
                let request = v1::ReconcileUserConfigRequest {
                    schema: res.schema.clone(),
                    name: res.name.clone(),
                    spec: res.spec.0.clone(),
                    children,
                    state: match res.state.clone() {
                        ResourceState::Unset => Some(
                            v1::reconcile_user_config_request::State::Unset(()),
                        ),
                        ResourceState::Set(state) => Some(
                            v1::reconcile_user_config_request::State::Ready(
                                state.0,
                            ),
                        ),
                    },
                };
                let response = client
                    .reconcile_user_config(request)
                    .await
                    .unwrap()
                    .into_inner();

                res.state = ResourceState::Set(State(response.state));
                let owner = Identity {
                    schema: res.schema.clone(),
                    name: res.name.clone(),
                };
                for to_create in response.children {
                    let id = Identity {
                        schema: to_create.schema.clone(),
                        name: to_create.name.clone(),
                    };
                    self.resources.insert(
                        id.clone(),
                        Resource::DynamicResource(DynamicResource {
                            schema: to_create.schema.clone(),
                            name: to_create.name.clone(),
                            owner: owner.clone(),
                            spec: Spec(to_create.spec),
                            state: ResourceState::Unset,
                        }),
                    );
                }
            }
            Resource::DynamicResource(res) => {
                let request = v1::ReconcileDynamicResourceRequest {
                    schema: res.schema.clone(),
                    name: res.name.clone(),
                    spec: res.spec.0.clone(),
                    owner: Some(v1::Identity {
                        schema: res.owner.schema.clone(),
                        name: res.owner.name.clone(),
                    }),
                    children,
                    state: match res.state.clone() {
                        ResourceState::Unset => Some(
                            v1::reconcile_dynamic_resource_request::State::Unset(()),
                        ),
                        ResourceState::Set(state) => Some(
                            v1::reconcile_dynamic_resource_request::State::Ready(
                                state.0,
                            ),
                        ),
                    },
                };
                let response = client
                    .reconcile_dynamic_resource(request)
                    .await
                    .unwrap()
                    .into_inner();

                res.state = ResourceState::Set(State(response.state));
                dbg!(&res.state);
                let owner = Identity {
                    schema: res.schema.clone(),
                    name: res.name.clone(),
                };
                for to_create in response.children {
                    let id = Identity {
                        schema: to_create.schema.clone(),
                        name: to_create.name.clone(),
                    };
                    self.resources.insert(
                        id.clone(),
                        Resource::DynamicResource(DynamicResource {
                            schema: to_create.schema.clone(),
                            name: to_create.name.clone(),
                            owner: owner.clone(),
                            spec: Spec(to_create.spec),
                            state: ResourceState::Unset,
                        }),
                    );
                }
            }
        }
    }
}
