use std::sync::Arc;
use std::time::Instant;

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
use serde_json::Value;
use tonic::{Request, Response, Status};

use crate::state::StateManager;

pub struct ApiServiceThing {
    pub sm: Arc<StateManager>,
}

#[tonic::async_trait]
impl ApiService for ApiServiceThing {
    async fn reconcile_now(
        &self,
        request: Request<ReconcileNowRequest>,
    ) -> Result<Response<ReconcileNowResponse>, Status> {
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
        _request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
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
        let req = request.into_inner();
        let resources: Vec<SubResourceCreate<Value>> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

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
        _request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        todo!()
    }

    async fn force_delete(
        &self,
        request: Request<ForceDeleteRequest>,
    ) -> Result<Response<ForceDeleteResponse>, Status> {
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
