use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordVerifier as _};
use cos_proto_api::v1::{
    ForceDeleteRequest,
    ForceDeleteResponse,
    GetResourceRequest,
    GetResourceResponse,
    ListResourcesRequest,
    ListResourcesResponse,
    PushConfigRequest,
    PushConfigResponse,
    ReconcileNowRequest,
    ReconcileNowResponse,
    ShutdownRequest,
    ShutdownResponse,
};
use cos_proto_api_server::v1::ApiService;
use cos_proto_reconciler::{Identity, Key, PrivateIdentity, SubResourceCreate};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::state::StateManager;

pub struct ApiServer {
    pub sm: Arc<StateManager>,
    pub config: Mutex<ApiConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub auth: ApiAuth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiAuth {
    None,
    Password(String),
}

impl ApiServer {
    async fn auth_or_fail<T>(&self, req: &Request<T>) -> Result<(), Status>
    where
        T: Send + Sync,
    {
        let cfg = self.config.lock().await;
        match &cfg.auth {
            ApiAuth::None => Ok(()),
            ApiAuth::Password(hash) => {
                let Some(password) = req.metadata().get("x-auth") else {
                    return Err(Status::unauthenticated("no password"));
                };

                let argon2 = Argon2::default();
                let hash = PasswordHash::new(hash)
                    .map_err(|v| Status::unauthenticated(format!("{v}")))?;
                argon2
                    .verify_password(password.as_bytes(), &hash)
                    .map_err(|v| Status::unauthenticated(format!("{v}")))
            }
        }
    }
}

#[tonic::async_trait]
impl ApiService for ApiServer {
    async fn reconcile_now(
        &self,
        request: Request<ReconcileNowRequest>,
    ) -> Result<Response<ReconcileNowResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();

        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        self.sm.queue.schedule_at(key, Instant::now()).await;

        Ok(Response::new(ReconcileNowResponse {
            raw: vec![],
        }))
    }

    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let guard = self.sm.resources.read().await;
        let resources = guard.values().cloned().collect_vec();
        drop(guard);

        Ok(Response::new(ListResourcesResponse {
            raw: serde_json::to_vec(&resources)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn get_resource(
        &self,
        request: Request<GetResourceRequest>,
    ) -> Result<Response<GetResourceResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;
        let guard = self.sm.resources.read().await;
        let resources = guard.get(&key).cloned();
        drop(guard);

        Ok(Response::new(GetResourceResponse {
            raw: serde_json::to_vec(&resources)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn push_config(
        &self,
        request: Request<PushConfigRequest>,
    ) -> Result<Response<PushConfigResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let resources: Vec<SubResourceCreate<Value>> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        let Some(cfg) = resources.iter().find(|v| {
            v.id.key()
                == &Key {
                    schema: "api".to_owned(),
                    name: None,
                }
        }) else {
            return Err(Status::invalid_argument(
                "api config must be present",
            ));
        };

        let cfg: ApiConfig = serde_json::from_value(cfg.spec.clone())
            .map_err(|err| Status::from_error(err.into()))?;

        {
            let mut guard = self.config.lock().await;
            *guard = cfg;
        }

        let mut guard = self.sm.resources.write().await;
        let clients = self.sm.clients.read().await;

        let old_keys = guard
            .values()
            .filter_map(|v| match &v.id {
                Identity::Private(PrivateIdentity::Static(k)) => Some(k),
                Identity::Private(PrivateIdentity::Dynamic(_))
                | Identity::Shared(_) => None,
            })
            .cloned()
            .collect();

        let updated_resources = resources
            .into_iter()
            .map(|v| (v.id.key().clone(), v))
            .collect();

        StateManager::bulk_upsert(
            &clients,
            &self.sm.queue,
            &mut guard,
            old_keys,
            updated_resources,
        )
        .await
        .map_err(|err| Status::from_error(err.into()))?;
        drop(guard);

        Ok(Response::new(PushConfigResponse { raw: vec![] }))
    }

    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        self.auth_or_fail(&request).await?;
        todo!()
    }

    async fn force_delete(
        &self,
        request: Request<ForceDeleteRequest>,
    ) -> Result<Response<ForceDeleteResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        tracing::info!(%key, "forced deletion");

        let mut guard = self.sm.resources.write().await;
        let _ = guard.remove(&key);
        drop(guard);

        Ok(Response::new(ForceDeleteResponse { raw: vec![] }))
    }
}
