mod crud;
mod model;
mod reconciliation;

use std::collections::HashMap;

use cos_api_reconciler_client::proto::v1::ReconcilerServiceClient;
use cos_api_shared::{Identity, Resource};
pub use model::*;
use tokio_util::time::DelayQueue;
use tonic::transport::{Channel, Endpoint};

pub struct StateManager {
    pub resources: HashMap<Identity, Resource<Payload>>,
    clients: HashMap<String, ReconcilerServiceClient<Channel>>,
    reconciliation_queue: DelayQueue<Identity>,
}

impl StateManager {
    pub fn new() -> Self {
        let conn = Endpoint::from_static("http://[::1]:50052").connect_lazy();
        let client = ReconcilerServiceClient::new(conn);

        // TODO: Add proper reconciler registration
        Self {
            resources: HashMap::default(),
            clients: HashMap::from([
                (
                    "contaienros/LinkConfig".to_string(),
                    client.clone(),
                ),
                ("contaienros/LinkSpec".to_string(), client.clone()),
                (
                    "contaienros/AddressSpec".to_string(),
                    client.clone(),
                ),
                (
                    "contaienros/RouteSpec".to_string(),
                    client,
                ),
            ]),
            reconciliation_queue: DelayQueue::default(),
        }
    }

    fn get_client_for_id(
        &self,
        id: &Identity,
    ) -> Option<ReconcilerServiceClient<Channel>> {
        self.clients.get(id.schema()).cloned()
    }
}
