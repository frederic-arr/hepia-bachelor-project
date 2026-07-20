use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, anyhow, bail};
use cos_proto_reconciler::v1::{self, ValidateRequest};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
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
use serde_json::Value;
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
    clients: RwLock<Clients>,
    resources: RwLock<Resources>,
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
            let last = *self.last_state_change.lock().await;

            let elapsed = last.duration_since(self.init_time);
            tracing::debug!(?elapsed, "first start -> current state");
            let elapsed = last.elapsed();
            tracing::debug!(?elapsed, "time since last state change");

            if cancellation_token.is_cancelled() {
                tracing::trace!("reconciliation loop was cancelled");
                break;
            }

            self.reconciliation_loop_tick(cancellation_token).await;
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
                        tracing::error!("{err:#}");
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
        // resource.dependents = dependents;

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

        // TODO:
        // resource.persistent_state = reconciled.persistent_state;
        // resource.ephemeral_state = reconciled.ephemeral_state;

        resource.state = reconciled.state;
        let notify_dependents = matches!(
            (&resource.status, &reconciled.status),
            (
                Status::NotReady | Status::Unknown | Status::Error(_),
                Status::Ready | Status::Done,
            ) | (Status::Ready, Status::Done)
        );

        tracing::debug!("notifying dependents of state change");
        if notify_dependents {
            let mut last = self.last_state_change.lock().await;
            *last = Instant::now();
            drop(last);
            self.queue
                .schedule_at_bulk(
                    dependents.iter().map(Identity::key).cloned().collect(),
                    Instant::now(),
                )
                .await;
        }

        if let Status::Error(StatusError::Other(err)) = &reconciled.status {
            tracing::error!("{err}");
        }

        resource.status = reconciled.status.clone();

        let old_children = resource
            .children
            .iter()
            .map(cos_proto_reconciler::Identity::key)
            .cloned()
            .collect();

        resource.children =
            reconciled.children.iter().map(|v| &v.id).cloned().collect();
        resource.dependencies.clone_from(&reconciled.dependencies);

        let id = resource.id.clone();
        Self::bulk_upsert(
            &*self.clients.read().await,
            &self.queue,
            &mut state,
            old_children,
            reconciled
                .children
                .into_iter()
                .map(|v| (v.id.key().clone(), v))
                .collect(),
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

        tracing::trace!("reconciliation succesfull");
        if reconciled.status == Status::Done {
            return Ok(None);
        }

        Ok(Some(
            Instant::now() + DEFAULT_RECONCILIATION_TIMER,
        ))
    }

    #[expect(clippy::too_many_lines, reason = "TODO")]
    pub async fn bulk_upsert(
        clients: &Clients,
        queue: &Queue<Key>,
        self_resources: &mut Resources,
        old_keys: HashSet<Key>,
        mut updated_resources: HashMap<Key, SubResourceCreate<Value>>,
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

        let mut added_fut = JoinSet::new();
        let mut modified_fut = JoinSet::new();

        for resource in added {
            let mut client =
                clients.get(resource.id.schema()).cloned().ok_or_else(
                    || anyhow!("no clients for {}", resource.id.schema()),
                )?;

            added_fut.spawn(async move {
                let request = tonic::Request::new(ValidateRequest {
                    raw: serde_json::to_vec(&(
                        resource.clone(),
                        None::<Resource<Value, Value, Value>>,
                    ))?,
                });

                let response = client.validate(request).await?.into_inner();
                let response: ValidateResponse<Value> =
                    serde_json::from_slice(&response.raw)?;

                anyhow::Ok((resource, response))
            });
        }

        for resource in modified {
            let mut client =
                clients.get(resource.id.schema()).cloned().ok_or_else(
                    || anyhow!("no clients for {}", resource.id.schema()),
                )?;

            modified_fut.spawn(async move {
                let request = tonic::Request::new(ValidateRequest {
                    raw: serde_json::to_vec(&(
                        resource.clone(),
                        None::<Resource<Value, Value, Value>>,
                    ))?,
                });

                let response = client.validate(request).await?.into_inner();
                let response: ValidateResponse<Value> =
                    serde_json::from_slice(&response.raw)?;

                anyhow::Ok((resource, response))
            });
        }

        let (added, modified) =
            tokio::join!(added_fut.join_all(), modified_fut.join_all());

        let added = added.into_iter().collect::<Result<Vec<_>, _>>()?;
        let modified = modified.into_iter().collect::<Result<Vec<_>, _>>()?;

        for resource in removed {
            let Some(entry) = self_resources.get_mut(&resource) else {
                continue;
            };

            entry.phase = Phase::Teardown;
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
        let scheduled = added
            .iter()
            .chain(modified.iter())
            .filter(|(_, b)| {
                b.dependencies
                    .iter()
                    .map(|v| self_resources.get(v.key()))
                    .all(|v| {
                        v.is_some_and(|v| {
                            matches!(v.status, Status::Done | Status::Ready)
                        })
                    })
            })
            .map(|(v, _)| v.id.key())
            .cloned()
            .collect();

        queue.schedule_at_bulk(scheduled, Instant::now()).await;
        Ok(())
    }
}
