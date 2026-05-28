mod crud;
// mod model;
mod reconciliation;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use cos_api_reconciler_client::proto::v1::ReconcilerServiceClient;
// pub use model::*;
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
                    ".containeros.net.link-config".to_string(),
                    client.clone(),
                ),
                (
                    ".containeros.net.link-spec".to_string(),
                    client.clone(),
                ),
                (
                    ".containeros.net.address-spec".to_string(),
                    client.clone(),
                ),
                (
                    ".containeros.net.route-spec".to_string(),
                    client.clone(),
                ),
                (
                    ".containeros.containers.container-spec".to_string(),
                    client2.clone(),
                ),
            ]),
            scheduled_identities: HashMap::default(),
            reconciliation_queue: DelayQueue::default(),
        }
    }

    fn get_client_for_id(
        &self,
        id: &Identity,
    ) -> Option<ReconcilerServiceClient<Channel>> {
        self.clients.get(id.schema()).cloned()
    }

    // fn tree_of<'a>(&'a self, id: &Identity) -> Vec<&'a Resource> {
    // let mut queue = VecDeque::from([id]);
    // let mut tree = Vec::with_capacity(25);
    // while let Some(id) = queue.pop_front() {
    // let Some(res) = self.resource_read(id) else {
    // continue;
    // };
    // tree.push(res);
    // queue.extend(res.children());
    // }
    //
    // tree
    // }
    //
    // fn for_each_in_tree(
    // &mut self,
    // id: &Identity,
    // mut before: impl FnMut(&mut Resource),
    // mut after: impl FnMut(&mut Resource),
    // ) {
    // let mut queue = VecDeque::from([id.clone()]);
    //
    // while let Some(id) = queue.pop_front() {
    // let Some(res) = self.resource_read_mut(&id) else {
    // continue;
    // };
    //
    // before(res);
    //
    // let children: Vec<Identity> =
    // res.children().iter().cloned().collect();
    //
    // after(res);
    //
    // queue.extend(children);
    // }
    // }
    //
    // fn mark_for_deletion(&mut self, id: &Identity) {
    // todo!()
    // let Some(res) = self.resource_read_mut(&id) else {
    //     return;
    // };
    //
    // *res.status_mut() = ResourceStatus::Deleting;
    // for child in res.children().clone() {
    //     self.mark_for_deletion(&child);
    // }
    // }
    //
    // fn try_delete(&mut self, id: &Identity) -> Result<(), ()> {
    // todo!()
    // let Entry::Occupied(mut e) = self.resources.entry(id.clone()) else {
    //     return Ok(());
    // };
    //
    // if e.get().status() != &ResourceStatus::Deleting {
    //     return Err(());
    // }
    //
    // if e.get().children().is_empty() {
    //     let v = e.remove();
    //     if let Some(parent) = self.resource_read_mut(v.id()) {
    //         parent.children_mut().remove(v.id());
    //     }
    //
    //     return Ok(());
    // }
    //
    // return Err(());
    // }

    // fn collect_deletion(&mut self, id: &Identity) -> bool {
    //     let Entry::Occupied(mut e) = self.resources.entry(id.clone()) else {
    //         return true;
    //     };

    //     if e.get().status() != &ResourceStatus::Deleting {
    //         *e.get_mut().status_mut() = ResourceStatus::Deleting;
    //         return false;
    //     }

    //     if e.get().children().is_empty() {
    //         let v = e.remove();
    //         if let Some(parent) = self.resource_read_mut(v.id()) {
    //             parent.children_mut().remove(v.id());
    //         }

    //         return true;
    //     }

    //     let children = e.get().children().clone();
    //     let mut all_true = true;
    //     for child in children {
    //         all_true |= self.collect_deletion(&child);
    //     }

    //     if !all_true {
    //         return false;
    //     }

    //     let v = e.remove();
    //     if let Some(parent) = self.resource_read_mut(v.id()) {
    //         parent.children_mut().remove(v.id());
    //     }

    //     return true;
    // }

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
        self.scheduled_identities
            .entry(id)
            .and_modify(|existing| {
                let existing_when =
                    self.reconciliation_queue.deadline(existing);
                match existing_when.cmp(&when) {
                    std::cmp::Ordering::Less => {
                        self.reconciliation_queue.reset_at(existing, when);
                    }
                    std::cmp::Ordering::Equal => return,
                    std::cmp::Ordering::Greater => return,
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
        self.scheduled_identities
            .entry(id)
            .and_modify(|existing| {
                let existing_when =
                    self.reconciliation_queue.deadline(existing);
                match existing_when.cmp(&when) {
                    std::cmp::Ordering::Less => return,
                    std::cmp::Ordering::Equal => return,
                    std::cmp::Ordering::Greater => {
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
        self.schedule_reconcile_at_earliest(id, Instant::now() + when)
    }

    /// Like [`Self::schedule_reconcile_at_latest`] but executes offset by
    /// [`Instant::now`]
    fn schedule_reconcile_at_latest_in(
        &mut self,
        id: Identity,
        when: Duration,
    ) {
        self.schedule_reconcile_at_latest(id, Instant::now() + when)
    }
}
