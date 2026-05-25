#![feature(map_try_insert)]

mod state_manager;

use cos_api_shared::{Identity, Resource};
use cos_api_sysmgr::proto::v1;
use cos_api_sysmgr_server::proto::v1 as v1_svc;
use serde_json::json;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::{Request, Response, Status};

use crate::state_manager::{
    CreateConfig,
    CreateResource,
    Payload,
    StateManager,
};

struct SystemManagerInner {
    state_manager: StateManager,
}

pub struct SystemManagerService(RwLock<SystemManagerInner>);

impl SystemManagerInner {
    fn new() -> Self {
        Self {
            state_manager: StateManager::new(),
        }
    }
}

impl SystemManagerService {
    fn new() -> Self {
        Self(RwLock::new(SystemManagerInner::new()))
    }

    async fn read(&self) -> RwLockReadGuard<'_, SystemManagerInner> {
        self.0.read().await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, SystemManagerInner> {
        self.0.write().await
    }
}

#[tonic::async_trait]
impl v1_svc::SystemManagerService for SystemManagerService {
    async fn resource_create_dynamic(
        &self,
        request: Request<v1::ResourceCreateDynamicRequest>,
    ) -> Result<Response<v1::ResourceCreateDynamicResponse>, Status> {
        let resource: CreateResource = request.into_inner().try_into().unwrap();

        let mut inner = self.write().await;
        inner
            .state_manager
            .resource_create(resource)
            .map_err(Status::internal)?;
        drop(inner);

        Ok(Response::new(
            v1::ResourceCreateDynamicResponse {},
        ))
    }

    async fn resource_read(
        &self,
        request: Request<v1::ResourceReadRequest>,
    ) -> Result<Response<v1::ResourceReadResponse>, Status> {
        let id: Identity = request.into_inner().id.try_into().unwrap();

        let inner = self.read().await;
        let resource = inner.state_manager.resource_read(&id).cloned();
        drop(inner);

        resource
            .ok_or_else(|| {
                Status::not_found(format!("resource {id} was not found"))
            })
            .map(|res| v1::ResourceReadResponse {
                resource: Some(res.try_into().unwrap()),
            })
            .map(Response::new)
    }

    async fn resource_list(
        &self,
        _request: Request<v1::ResourceListRequest>,
    ) -> Result<Response<v1::ResourceListResponse>, Status> {
        todo!()
    }

    async fn resource_update_dynamic_spec(
        &self,
        _request: Request<v1::ResourceUpdateDynamicSpecRequest>,
    ) -> Result<Response<v1::ResourceUpdateDynamicSpecResponse>, Status> {
        todo!()
    }

    async fn resource_update_state(
        &self,
        _request: Request<v1::ResourceUpdateStateRequest>,
    ) -> Result<Response<v1::ResourceUpdateStateResponse>, Status> {
        todo!()
    }

    async fn resource_delete_dynamic(
        &self,
        _request: Request<v1::ResourceDeleteDynamicRequest>,
    ) -> Result<Response<v1::ResourceDeleteDynamicResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50051".parse().unwrap();
    let system_manager = SystemManagerService::new();
    let mut sm = system_manager.0.write().await;

    let spec = json!({
        "state": "down",
    });

    let spec = rmp_serde::to_vec(&spec).unwrap();
    let cfg = CreateConfig {
        id: Identity::new(
            "contaienros/LinkConfig".to_string(),
            "dummy0".to_string(),
        ),
        spec: spec.into(),
    };

    sm.state_manager.config_create(cfg.clone()).unwrap();

    let se = serde_json::to_string_pretty(
        &sm.state_manager.resources.values().collect::<Vec<_>>(),
    )
    .unwrap();

    let de: Vec<Resource<Payload>> = serde_json::from_str(&se).unwrap();
    let _se2 = serde_json::to_string_pretty(&de).unwrap();

    let se = serde_json::to_string_pretty(&cfg).unwrap();
    let de: CreateConfig = serde_json::from_str(&se).unwrap();
    let se2 = serde_json::to_string_pretty(&de).unwrap();

    println!("{se2}");
    assert_eq!(se, se2);

    // sm.state_manager.reconciliation_loop().await;

    // println!("GreeterServer listening on {addr}");

    // Server::builder()
    //     .add_service(v1_svc::SystemManagerServiceServer::new(
    //         system_manager,
    //     ))
    //     .serve(addr)
    //     .await?;

    drop(sm);
    Ok(())
}
