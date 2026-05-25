use std::time::Duration;

use cos_api_reconciler::proto::v1::ReconcileResourceRequest;
use cos_api_shared::Identity;
use tokio_stream::StreamExt;

use crate::state_manager::StateManager;

impl StateManager {
    pub async fn reconciliation_loop(&mut self) {
        while let Some(exp) = self.reconciliation_queue.next().await {
            self.reconciliation_tick(exp.into_inner()).await;
        }
    }

    async fn reconciliation_tick(&mut self, id: Identity) {
        // TODO: Use the correct reconciler

        dbg!(&id);
        let resource = self.resources.get_mut(&id).unwrap();
        let mut client = self.reconcilers.get(&"".to_string()).unwrap().clone();
        let res = client
            .reconcile_resource(ReconcileResourceRequest {
                resource: Some(resource.clone().try_into().unwrap()),
                additional_resources: vec![],
            })
            .await
            .unwrap();

        resource
            .state_opt_mut()
            .replace(res.into_inner().state.into());
        self.reconciliation_queue.insert(id, Duration::from_secs(5));
        dbg!(&resource);
    }
}
