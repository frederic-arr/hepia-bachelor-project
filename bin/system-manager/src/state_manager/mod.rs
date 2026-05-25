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
    resources: HashMap<Identity, Resource<Payload>>,
    reconcilers: HashMap<String, ReconcilerServiceClient<Channel>>,
    reconciliation_queue: DelayQueue<Identity>,
}

impl StateManager {
    pub fn new() -> Self {
        let conn = Endpoint::from_static("http://[::1]:50052").connect_lazy();
        let client = ReconcilerServiceClient::new(conn);

        Self {
            resources: HashMap::default(),
            reconcilers: HashMap::from([("".to_string(), client)]),
            reconciliation_queue: DelayQueue::default(),
        }
    }
}
