use std::collections::hash_map::Entry;
use std::time::Duration;

use cos_api_reconciler::proto::v1::{
    ReconcileDeleteRequest,
    ReconcileResourceRequest,
};
use cos_api_shared::{Identity, Resource, ResourceStatus};
use invariant_macros::invariant_violation;
use tokio::time::Instant;
use tokio_stream::StreamExt;

use crate::state_manager::{CreateDynamicResource, Payload, StateManager};

impl StateManager {
    pub async fn reconciliation_loop(&mut self) {
        // while let Some(exp) = self.reconciliation_queue.next().await {
        // }
        loop {
            let ids = self.resources.keys().cloned().collect::<Vec<_>>();
            for id in ids {
                self.reconciliation_tick(&id).await;
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    async fn reconciliation_tick(&mut self, id: &Identity) {
        tracing::info!("reconciliation attempt of {id}");

        let mut client = self.get_client_for_id(&id).unwrap();

        let Entry::Occupied(mut e) = self.resources.entry(id.clone()) else {
            invariant_violation!(
                "reconciliation scheduled on a non-existing resource: {id}"
            );
        };

        let current = e.get_mut();

        // TODO: Should be refactored as it is quite ugly right now
        if current.status() == &ResourceStatus::Deleting {
            if !current.children().is_empty() {
                return;
            }

            let res = client
                .reconcile_delete(ReconcileDeleteRequest {
                    resource: Some(current.clone().try_into().unwrap()),
                })
                .await
                .unwrap();

            if !res.into_inner().deleted {
                return;
            }

            let Resource::Dynamic(res) = e.remove() else {
                // We are removing a UserConfig. Since it's the "root" of any
                // resource, there's nothing else to do
                return;
            };

            let Some(owner) = self.resources.get_mut(res.owner()) else {
                invariant_violation!(
                    "dynamic resource {} owner's {} does not exist",
                    res.meta().id(),
                    res.owner(),
                );
            };

            owner.children_mut().remove(res.meta().id());
            if owner.children().is_empty() {
                self.schedule_reconcile_at_earliest_in(
                    res.owner().clone(),
                    Duration::from_secs(1),
                );
            }

            return;
        }

        let res = client
            .reconcile_resource(ReconcileResourceRequest {
                resource: Some(current.clone().try_into().unwrap()),
                additional_resources: vec![],
            })
            .await
            .unwrap();

        let res = res.into_inner();
        let creations =
            res.created.into_iter().map(|res| CreateDynamicResource {
                id: res.id.try_into().unwrap(),
                owner: id.clone(),
                spec: res.spec,
            });

        current.state_opt_mut().replace(res.state.into());

        self.resource_dynamic_create_bulk(creations.collect(), None)
            .unwrap();

        for deleted in res.deleted {
            self.mark_for_deletion(&deleted.try_into().unwrap());
        }

        // TODO: Handle updates

        self.schedule_reconcile_at_latest_in(
            id.clone(),
            Duration::from_secs(5),
        );
    }
}
