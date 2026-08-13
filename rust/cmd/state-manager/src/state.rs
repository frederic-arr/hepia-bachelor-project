use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, anyhow, bail};
use cos_proto_reconciler::v1::{self, ValidateRequest};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    PrivateIdentity,
    Resource,
    ResourceResponse,
    Status,
    StatusError,
    SubResourceCreate,
    TerminalResource,
    ValidateResponse,
};
use cos_proto_reconciler_client::v1::ReconcilerServiceClient;
use itertools::Itertools as _;
use linux_utils::is_maintenance;
use rustix::fs::sync;
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tonic::IntoRequest as _;
use tonic::transport::Channel;

use crate::queue::Queue;
use crate::timeout::Timeout as _;

type Clients = HashMap<String, ReconcilerServiceClient<Channel>>;
type Resources = HashMap<Key, TerminalResource<Value, Value, Value>>;

const DEFAULT_RECONCILIATION_TIMER: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct StateManager {
    pub clients: RwLock<Clients>,
    pub resources: RwLock<Resources>,
    pub queue: Queue<Key>,
    init_time: Instant,
    last_state_change: Mutex<Instant>,
}

impl StateManager {
    pub async fn new(
        clients: Clients,
        resources: Resources,
        queue: Queue<Key>,
    ) -> Self {
        Self {
            clients: RwLock::new(clients),
            resources: RwLock::new(resources),
            queue,
            init_time: Instant::now(),
            last_state_change: Mutex::new(Instant::now()),
        }
    }

    pub async fn reconciliation_loop(
        &self,
        cancellation_token: &CancellationToken,
    ) {
        loop {
            if cancellation_token.is_cancelled() {
                tracing::trace!("reconciliation loop was cancelled");
                break;
            }

            self.reconciliation_loop_tick(cancellation_token).await;
            if !is_maintenance()
                && let Err(err) = self.serialize_bundle().await
            {
                cancellation_token.cancel();
                tracing::error!("failed to serialize to disk: {err}");
                return;
            }
        }
    }

    async fn reconciliation_loop_tick(
        &self,
        cancellation_token: &CancellationToken,
    ) {
        tracing::trace!("reconciliation tick");
        let expired = tokio::select! {
            v = self.queue.drain_expired() => v,
            () = cancellation_token.cancelled() => {
                tracing::trace!("reconciliation tick was cancelled");
                return
            }
        }
        .into_iter()
        .flat_map(|(_, v)| v)
        .collect_vec();

        tracing::trace!(
            "new batch of reconciliation with {} keys",
            expired.len()
        );

        for key in expired {
            if cancellation_token.is_cancelled() {
                tracing::trace!("reconciliation batch was cancelled");
                return;
            }

            let when =
                match self.reconcile(key.clone(), cancellation_token).await {
                    Ok(when) => when,
                    Err(err) => {
                        tracing::error!("error while reconciling: {err:#}");
                        Some(Instant::now() + DEFAULT_RECONCILIATION_TIMER)
                    }
                };

            if let Some(when) = when {
                self.queue.schedule_at(key, when).await;
            }
        }
    }

    #[expect(clippy::too_many_lines, reason = "TODO")]
    async fn reconcile(
        &self,
        key: Key,
        cancellation_token: &CancellationToken,
    ) -> Result<Option<Instant>> {
        if key.schema == "api" || key.schema == "install" {
            return Ok(None);
        }

        tracing::info!("attempting to reconcile {key}");
        let max_duration = Duration::from_secs(5);
        let deadline = Instant::now() + max_duration;

        let Ok(mut state) = self.resources.write().timeout_at(deadline).await
        else {
            bail!("failed to aquire write lock during reconciliation");
        };

        let resource = state.get(&key).context("resource not found")?;
        let raw = Resource::<Value, Value, Value> {
            id: resource.id.clone(),
            phase: resource.phase.clone(),
            status: resource.status.clone(),
            spec: resource.spec.clone(),
            derived_spec: resource.derived_spec.clone(),
            state: resource.state.clone(),
            children: resource
                .children
                .iter()
                .filter_map(|id| state.get(id.key()))
                .cloned()
                .collect(),
            dependencies: resource
                .dependencies
                .iter()
                .filter_map(|id| state.get(id.key()))
                .cloned()
                .collect(),
            dependents: resource
                .dependents
                .iter()
                .filter_map(|id| state.get(id.key()))
                .cloned()
                .collect(),
        };

        let dependents = state
            .values()
            .filter(|v| {
                v.dependencies.contains(&resource.id)
                    || v.children.contains(&resource.id)
            })
            .map(|v| &v.id)
            .cloned()
            .collect_vec();

        let resource = state.get_mut(&key).context("resource not found")?;

        let Some(mut client) =
            self.clients.read().await.get(&key.schema).cloned()
        else {
            tracing::error!("no clients for {key}");
            resource.status = Status::Error(StatusError::NoClient);
            return Ok(None);
        };

        let Ok(raw) = serde_json::to_vec(&raw) else {
            tracing::error!("unable to serialize {key}");
            resource.status = Status::Error(StatusError::Internal);
            return Ok(None);
        };

        let mut request = v1::ReconcileRequest { raw }.into_request();
        request.set_timeout(max_duration);

        // Although reconciliation should be idempotent, we don't know how the
        // reconciler will behave with a partially correct state. This is the
        // last "safe" cancellation point.
        // Normally a task can be aborted at *any* await point, but we launch
        // them in a way that make them "unabortable".
        if cancellation_token.is_cancelled() {
            tracing::trace!("reconciliation was cancelled");
            return Ok(None);
        }

        let Ok(response) = client.reconcile(request).timeout_at(deadline).await
        else {
            tracing::error!("reconciliation of {key} timed-out");
            resource.status = Status::Error(StatusError::TimedOut);
            return Ok(None);
        };

        let response = match response {
            Ok(response) => response.into_inner(),
            Err(err) => {
                let err = if err.code() == tonic::Code::DeadlineExceeded {
                    tracing::error!("reconciliation of {key} timed-out");
                    StatusError::TimedOut
                } else {
                    tracing::error!(
                        "an unexpected error happend while reconciling {key}: \
                         {}",
                        err.message(),
                    );
                    StatusError::Transport(err.message().to_owned())
                };
                resource.status = Status::Error(err);
                return Ok(None);
            }
        };

        let Ok(reconciled) =
            serde_json::from_slice::<ResourceResponse<Value>>(&response.raw)
        else {
            tracing::error!("failed to deserialize response for {key}");
            resource.status = Status::Error(StatusError::Invalid);
            return Ok(None);
        };

        if let Status::Error(StatusError::Other(err)) = &reconciled.status {
            tracing::error!("remote error: {err}");
        } else {
            tracing::info!(status = ?reconciled.status, key = %key, "reconciled resource");
        }

        let removed_deps = resource
            .dependencies
            .difference(&reconciled.dependencies)
            .cloned()
            .collect_vec();

        let added_deps = reconciled
            .dependencies
            .difference(&resource.dependencies)
            .cloned()
            .collect_vec();

        resource.state = reconciled.state;
        let notify_dependents = matches!(
            (&resource.status, &reconciled.status),
            (
                Status::NotReady | Status::Unknown | Status::Error(_),
                Status::Ready | Status::Done,
            ) | (Status::Ready, Status::Done)
        );

        resource.status = reconciled.status.clone();

        let old_children: HashSet<_> = resource
            .children
            .iter()
            .map(cos_proto_reconciler::Identity::key)
            .cloned()
            .collect();

        let old_shared_deps: HashSet<_> = resource
            .dependencies
            .iter()
            .filter(|id| matches!(id, Identity::Shared(_)))
            .map(|id| id.key().clone())
            .collect();

        let old_keys = old_children.union(&old_shared_deps).cloned().collect();

        resource.children =
            reconciled.children.iter().map(|v| &v.id).cloned().collect();
        resource.dependencies.clone_from(&reconciled.dependencies);

        let id = resource.id.clone();
        if notify_dependents {
            let mut last = self.last_state_change.lock().await;
            *last = Instant::now();
            drop(last);

            let dependents =
                dependents.iter().map(Identity::key).cloned().collect_vec();

            let keys = state
                .values()
                .filter(|v| dependents.contains(v.id.key()))
                .map(|v| {
                    (
                        SubResourceCreate::<Value> {
                            id: v.id.clone(),
                            spec: v.spec.clone(),
                        },
                        ValidateResponse::<Value> {
                            derived_spec: v.derived_spec.clone(),
                            children: vec![],
                            dependencies: v.dependencies.clone(),
                        },
                    )
                })
                .collect_vec();

            Self::schedule_available(keys.iter(), &self.queue, &state, true)
                .await?;
        }

        Self::bulk_upsert(
            &*self.clients.read().await,
            &self.queue,
            &mut state,
            old_keys,
            reconciled
                .children
                .into_iter()
                .map(|v| (v.id.key().clone(), v))
                .chain(
                    reconciled
                        .dependencies
                        .iter()
                        .filter(|v| matches!(v, Identity::Shared(_)))
                        .map(|v| {
                            (
                                v.key().clone(),
                                SubResourceCreate {
                                    id: v.clone(),
                                    spec: json!({
                                        "name": &v.key().name
                                    }),
                                },
                            )
                        }),
                )
                .collect(),
            true,
        )
        .await?;

        for dep in removed_deps {
            let Some(entry) = state.get_mut(dep.key()) else {
                continue;
            };

            entry.dependents.remove(&id);
        }

        for dep in added_deps {
            let Some(entry) = state.get_mut(dep.key()) else {
                continue;
            };

            entry.dependents.insert(id.clone());
        }

        if reconciled.status == Status::Deleted {
            tracing::info!(key = %key, "resource deleted");
            state.remove(&key);
            return Ok(None);
        }

        tracing::trace!("reconciliation succesfull");
        if reconciled.status == Status::Done {
            return Ok(None);
        }

        Ok(Some(
            Instant::now() + DEFAULT_RECONCILIATION_TIMER,
        ))
    }

    pub async fn bulk_validate(
        clients: &Clients,
        self_resources: &Resources,
        resources: Vec<SubResourceCreate<Value>>,
    ) -> Result<Vec<(SubResourceCreate<Value>, ValidateResponse<Value>)>> {
        let mut requests = JoinSet::new();
        for resource in resources {
            if resource.id.schema() == "api"
                || resource.id.schema() == "install"
            {
                requests.spawn(async {
                    let response = ValidateResponse::<Value> {
                        derived_spec: Value::Null,
                        children: vec![],
                        dependencies: HashSet::new(),
                    };

                    Ok((resource, response))
                });
                continue;
            }

            let mut client =
                clients.get(resource.id.schema()).cloned().ok_or_else(
                    || anyhow!("no clients for {}", resource.id.schema()),
                )?;

            let existing =
                self_resources.get(resource.id.key()).cloned().map(|v| {
                    Resource {
                        id: v.id,
                        phase: v.phase,
                        status: v.status,
                        spec: v.spec,
                        derived_spec: v.derived_spec,
                        state: v.state,
                        children: v
                            .children
                            .iter()
                            .filter_map(|k| self_resources.get(k.key()))
                            .cloned()
                            .collect(),
                        dependencies: v
                            .dependencies
                            .iter()
                            .filter_map(|k| self_resources.get(k.key()))
                            .cloned()
                            .collect(),
                        dependents: v
                            .dependents
                            .iter()
                            .filter_map(|k| self_resources.get(k.key()))
                            .cloned()
                            .collect(),
                    }
                });

            requests.spawn(async move {
                tracing::info!(key = %resource.id, "validating resource");

                let request = tonic::Request::new(ValidateRequest {
                    raw: serde_json::to_vec(&(resource.clone(), existing))?,
                });

                let response = client.validate(request).await?.into_inner();
                let response: ValidateResponse<Value> =
                    serde_json::from_slice(&response.raw)?;

                anyhow::Ok((resource, response))
            });
        }

        let requests = requests.join_all().await;
        requests.into_iter().collect::<Result<Vec<_>, _>>()
    }

    pub async fn bulk_upsert(
        clients: &Clients,
        queue: &Queue<Key>,
        self_resources: &mut Resources,
        old_keys: HashSet<Key>,
        mut updated_resources: HashMap<Key, SubResourceCreate<Value>>,
        do_schedule: bool,
    ) -> Result<()> {
        let mut resources: HashMap<_, _> = self_resources
            .iter()
            .filter(|(k, _)| old_keys.contains(k))
            .collect();

        let added: Vec<_> = updated_resources
            .extract_if(|k, _| !resources.contains_key(k))
            .map(|(_, v)| v)
            .collect();

        let removed: Vec<_> = resources
            .extract_if(|k, _| !updated_resources.contains_key(k))
            .map(|(k, _)| k)
            .cloned()
            .collect();

        let modified: Vec<_> = updated_resources
            .extract_if(|k, v| {
                resources.get(k).is_some_and(|a| a.spec != v.spec)
            })
            .map(|(_, v)| v)
            .collect();

        drop(updated_resources);

        let added_fut = Self::bulk_validate(clients, self_resources, added);
        let modified_fut =
            Self::bulk_validate(clients, self_resources, modified);

        let (added, modified) = tokio::try_join!(added_fut, modified_fut)?;

        let mut scheduled_removal = vec![];
        for resource in removed {
            let scheduled = Self::mark_removed(&resource, self_resources);
            scheduled_removal.extend(scheduled);
        }

        for (resource, response) in added.clone() {
            self_resources.insert(
                resource.id.key().clone(),
                TerminalResource {
                    id: resource.id,
                    phase: Phase::Running,
                    status: Status::Unknown,
                    spec: resource.spec,
                    derived_spec: response.derived_spec,
                    state: None,
                    children: HashSet::new(),
                    dependencies: response.dependencies,
                    dependents: HashSet::new(),
                },
            );
        }

        for (resource, response) in modified.clone() {
            let Some(entry) = self_resources.get_mut(resource.id.key()) else {
                continue;
            };

            entry.status = Status::Unknown;
            entry.spec = resource.spec;
            entry.derived_spec = response.derived_spec;
            entry.dependencies = response.dependencies;
        }

        queue
            .schedule_at_bulk(
                scheduled_removal.into_iter().collect(),
                Instant::now(),
            )
            .await;

        Self::schedule_available(
            added.iter().chain(modified.iter()),
            queue,
            self_resources,
            do_schedule,
        )
        .await
    }

    pub fn mark_removed(
        key: &Key,
        resources: &mut HashMap<Key, TerminalResource<Value, Value, Value>>,
    ) -> Vec<Key> {
        let Some(resource) = resources.get_mut(key) else {
            return vec![];
        };

        let no_inbound_deps = resource.dependents.is_empty();
        let no_children = resource.children.is_empty();
        let mut queued = vec![];
        if no_children && no_inbound_deps {
            resource.phase = Phase::Deleting;
            queued.push(key.clone());
        } else {
            resource.phase = Phase::PendingDeletion;
        }

        for child in resource.children.clone() {
            let keys = Self::mark_removed(child.key(), resources);
            queued.extend(keys);
        }

        queued
    }

    pub async fn schedule_available<'aaa, I>(
        keys: I,
        queue: &Queue<Key>,
        self_resources: &'aaa Resources,
        do_schedule: bool,
    ) -> Result<()>
    where
        I: Iterator<
            Item = &'aaa (SubResourceCreate<Value>, ValidateResponse<Value>),
        >,
    {
        let scheduled: HashSet<_> = keys
            .filter(|(_, b)| {
                b.dependencies
                    .iter()
                    .map(|v| (v, self_resources.get(v.key())))
                    .all(|(k, v)| {
                        (matches!(k, Identity::Shared(_)) && v.is_none())
                            || v.is_some_and(|v| {
                                matches!(v.status, Status::Done | Status::Ready)
                            })
                    })
            })
            .map(|(v, _)| v.id.key())
            .cloned()
            .collect();

        if do_schedule {
            queue.schedule_at_bulk(scheduled, Instant::now()).await;
        }

        Ok(())
    }

    pub async fn to_bundle(&self) -> Result<Vec<u8>> {
        let guard = self.resources.read().await;
        let resources = &guard
            .values()
            .filter(|v| {
                !matches!(
                    v.id,
                    Identity::Private(PrivateIdentity::Ephemeral(_))
                )
            })
            .collect_vec();
        let v = serde_json::to_vec(resources)?;
        drop(guard);
        Ok(v)
    }

    pub async fn serialize_bundle(&self) -> Result<()> {
        let bundle = self.to_bundle().await?;
        std::fs::write("/config/bundle.json", bundle)?;
        sync();

        Ok(())
    }

    pub fn from_bundle(bundle: &[u8]) -> Result<Resources> {
        let resources: Vec<TerminalResource<Value, Value, Value>> =
            serde_json::from_slice(bundle)?;

        let data = resources.into_iter().map(|v| {
            let key = v.id.key().clone();
            let value = TerminalResource {
                phase: Phase::Running,
                status: Status::Unknown,
                ..v
            };

            (key, value)
        });

        Ok(data.collect())
    }
}
