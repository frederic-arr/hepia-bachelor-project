// mod crud;
// mod model;
mod reconciliation;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use cos_api_reconciler_client::proto::v1::ReconcilerServiceClient;
use tokio::time::Instant;
use tokio_util::time::{DelayQueue, delay_queue};
use tonic::transport::{Channel, Endpoint};

use crate::resources::{Identity, Resource};

pub struct StateManager {
    pub resources: HashMap<Identity, Resource>,
    clients: HashMap<String, ReconcilerServiceClient<Channel>>,

    scheduled_identities: HashMap<Identity, delay_queue::Key>,
    reconciliation_queue: DelayQueue<Identity>,
}

impl StateManager {
    pub fn new() -> Self {
        let conn = Endpoint::from_static("http://[::1]:50052").connect_lazy();
        let client = ReconcilerServiceClient::new(conn);

        let conn2 = Endpoint::from_static("http://[::1]:50053").connect_lazy();
        let client2 = ReconcilerServiceClient::new(conn2);

        // TODO: Add proper reconciler registration
        Self {
            resources: HashMap::default(),
            clients: HashMap::from([
                (
                    "config#containeros::net::link".to_string(),
                    client.clone(),
                ),
                ("res#containeros::net::link".to_string(), client),
            ]),
            scheduled_identities: HashMap::default(),
            reconciliation_queue: DelayQueue::default(),
        }
    }

    fn get_client_for_id(
        &self,
        id: &Identity,
    ) -> Option<ReconcilerServiceClient<Channel>> {
        self.clients.get(&id.schema).cloned()
    }

    fn get_scheduled_when(&self, id: &Identity) -> Option<Instant> {
        self.scheduled_identities
            .get(id)
            .map(|key| self.reconciliation_queue.deadline(key))
    }

    /// Schedules a reconciliation to happen at `when` at the earliest.
    /// If a reconciliation is already scheduled for before `when`, it is reset
    /// to happen at `when`. If a reconciliation is already scheduled for
    /// after `when`, it is maintained. Otherwise, it is scheduled.
    fn schedule_reconcile_at_earliest(&mut self, id: Identity, when: Instant) {
        use std::cmp::Ordering::{Equal, Greater, Less};

        self.scheduled_identities
            .entry(id)
            .and_modify(|existing| {
                let existing_when =
                    self.reconciliation_queue.deadline(existing);
                match existing_when.cmp(&when) {
                    Greater | Equal => (),
                    Less => {
                        self.reconciliation_queue.reset_at(existing, when);
                    }
                }
            })
            .or_insert_with_key(|id| {
                self.reconciliation_queue.insert_at(id.clone(), when)
            });
    }

    /// Schedules a reconciliation to happen at `when` at the latest.
    /// If a reconciliation is already scheduled for before `when`, it is
    /// maintained. If a reconciliation is already scheduled for after
    /// `when`, it is reset to happen at `when`. Otherwise, it is scheduled.
    fn schedule_reconcile_at_latest(&mut self, id: Identity, when: Instant) {
        use std::cmp::Ordering::{Equal, Greater, Less};

        self.scheduled_identities
            .entry(id)
            .and_modify(|existing| {
                let existing_when =
                    self.reconciliation_queue.deadline(existing);
                match existing_when.cmp(&when) {
                    Less | Equal => (),
                    Greater => {
                        self.reconciliation_queue.reset_at(existing, when);
                    }
                }
            })
            .or_insert_with_key(|id| {
                self.reconciliation_queue.insert_at(id.clone(), when)
            });
    }

    /// Like [`Self::schedule_reconcile_at_earliest`] but executes offset by
    /// [`Instant::now`]
    fn schedule_reconcile_at_earliest_in(
        &mut self,
        id: Identity,
        when: Duration,
    ) {
        self.schedule_reconcile_at_earliest(id, Instant::now() + when);
    }

    /// Like [`Self::schedule_reconcile_at_latest`] but executes offset by
    /// [`Instant::now`]
    fn schedule_reconcile_at_latest_in(
        &mut self,
        id: Identity,
        when: Duration,
    ) {
        self.schedule_reconcile_at_latest(id, Instant::now() + when);
    }
}
