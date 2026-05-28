use std::collections::hash_map::Entry;
use std::time::{Duration, SystemTime};

use cos_api_reconciler::proto::v1;
// use cos_api_reconciler::proto::v1::{
//     ReconcileDeleteRequest,
//     ReconcileResourceRequest,
// };
use cos_api_shared::proto::v1 as v1_shared;
use invariant_macros::invariant_violation;
use tokio::time::Instant;
use tokio_stream::StreamExt;

use crate::resources::{
    Identity,
    Resource,
    ResourceSpec,
    ResourceState,
    State,
};
use crate::state_manager::StateManager;

impl StateManager {
    pub async fn reconciliation_loop(&mut self) {
        // while let Some(exp) = self.reconciliation_queue.next().await {
        // }
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

        let Some(mut client) = self.get_client_for_id(&id) else {
            tracing::error!("no clients for {id}");
            return;
        };

        let Entry::Occupied(mut e) = self.resources.entry(id.clone()) else {
            invariant_violation!(
                "reconciliation scheduled on a non-existing resource: {id}"
            );
        };

        let mut current = e.get_mut();
        match &mut current {
            Resource::UserConfig(res) => todo!(),
            Resource::Dynamic(res) => {
                let id = v1_shared::Identity {
                    schema: res.meta.id.schema().clone(),
                    name: res.meta.id.name().clone(),
                };

                let spec = match &res.meta.spec {
                    ResourceSpec::Running { spec } => spec,
                    ResourceSpec::Draining { spec } => spec,
                    ResourceSpec::Deleting { spec } => spec,
                };

                match &res.meta.state {
                    ResourceState::Unset => {
                        let response = client
                            .create_dynamic_resource(
                                v1::CreateDynamicResourceRequest {
                                    id: Some(id),
                                    spec: spec.0.clone(),
                                },
                            )
                            .await
                            .unwrap();

                        let new_state = match response.into_inner().state.unwrap() {
                            v1::create_dynamic_resource_response::State::Ready(state) => {
                                ResourceState::Ready { state: State(state.state.clone()) }
                            },
                            v1::create_dynamic_resource_response::State::Error(state) => todo!()
                        };
                        res.meta.state = new_state;
                    }
                    ResourceState::Ready { state, .. } => {
                        let response = client
                            .reconcile_dynamic_resource(
                                v1::ReconcileDynamicResourceRequest {
                                    id: Some(id),
                                    spec: spec.0.clone(),
                                    state: Some(v1::reconcile_dynamic_resource_request::State::Ready(v1::reconcile_dynamic_resource_request::StateReady {
                                        state: state.0.clone(),
                                    }))
                                },
                            )
                            .await.unwrap();

                        let new_state = match response.into_inner().state.unwrap() {
                                v1::reconcile_dynamic_resource_response::State::Ready(state) => {
                                    ResourceState::Ready { state: State(state.state) }
                                },
                                v1::reconcile_dynamic_resource_response::State::Error(state) => todo!()
                            };
                        res.meta.state = new_state;
                    }
                    ResourceState::Error { state, .. } => todo!(),
                }
            }
        };

        // TODO: Should be refactored as it is quite ugly right now
        // if matches!(
        //     current.spec(),
        //     ResourceSpec::Draining(_) | ResourceSpec::Deleting(_)
        // ) {
        //     if !current.children().is_empty() {
        //         return;
        //     }

        //     let res = client
        //         .reconcile_delete(ReconcileDeleteRequest {
        //             resource: Some(current.clone().try_into().unwrap()),
        //         })
        //         .await
        //         .unwrap();

        //     if !res.into_inner().deleted {
        //         return;
        //     }

        //     let Resource::Dynamic(res) = e.remove() else {
        //         // We are removing a UserConfig. Since it's the "root" of any
        //         // resource, there's nothing else to do
        //         return;
        //     };

        //     let Some(owner) = self.resources.get_mut(res.owner()) else {
        //         invariant_violation!(
        //             "dynamic resource {} owner's {} does not exist",
        //             res.id(),
        //             res.owner(),
        //         );
        //     };

        //     owner.children_mut().remove(res.id());
        //     if owner.children().is_empty() {
        //         self.schedule_reconcile_at_earliest_in(
        //             res.owner().clone(),
        //             Duration::from_secs(1),
        //         );
        //     }

        //     return;
        // }

        // let res = client
        //     .reconcile_resource(ReconcileResourceRequest {
        //         resource: Some(current.clone().try_into().unwrap()),
        //         additional_resources: vec![],
        //     })
        //     .await
        //     .unwrap();

        // let res = res.into_inner();
        // let creations =
        //     res.created.into_iter().map(|res| CreateDynamicResource {
        //         id: res.id.try_into().unwrap(),
        //         owner: id.clone(),
        //         spec: res.spec,
        //     });

        // *current.state_mut() = ResourceState::Ready {
        //     state: res.state.into(),
        //     state_at: SystemTime::now(),
        // };

        // dbg!(&current);

        // self.resource_dynamic_create_bulk(creations.collect(), None)
        //     .unwrap();

        // for deleted in res.deleted {
        //     self.mark_for_deletion(&deleted.try_into().unwrap());
        // }

        // // TODO: Handle updates

        // self.schedule_reconcile_at_latest_in(
        //     id.clone(),
        //     Duration::from_secs(5),
        // );
    }
}
