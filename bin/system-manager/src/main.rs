#![feature(map_try_insert)]

mod state_manager;

use cos_api_shared::*;
use cos_api_sysmgr::proto::v1;
use cos_api_sysmgr_server::proto::v1 as v1_svc;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::state_manager::{CreateConfig, CreateResource, StateManager};

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
        request: Request<v1::ResourceListRequest>,
    ) -> Result<Response<v1::ResourceListResponse>, Status> {
        todo!()
    }

    async fn resource_update_dynamic_spec(
        &self,
        request: Request<v1::ResourceUpdateDynamicSpecRequest>,
    ) -> Result<Response<v1::ResourceUpdateDynamicSpecResponse>, Status> {
        todo!()
    }

    async fn resource_update_state(
        &self,
        request: Request<v1::ResourceUpdateStateRequest>,
    ) -> Result<Response<v1::ResourceUpdateStateResponse>, Status> {
        todo!()
    }

    async fn resource_delete_dynamic(
        &self,
        request: Request<v1::ResourceDeleteDynamicRequest>,
    ) -> Result<Response<v1::ResourceDeleteDynamicResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Link {
    pub name: String,
    pub state: LinkState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LinkState {
    Up,
    Down,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50051".parse().unwrap();
    let system_manager = SystemManagerService::new();
    let mut sm = system_manager.0.write().await;

    let spec = Link {
        name: "dummy0".to_string(),
        state: LinkState::Down,
    };

    let spec = rmp_serde::to_vec(&spec).unwrap();
    dbg!(&spec);
    // let spec2: rmpv::Value = rmp_serde::from_slice(&spec).unwrap();
    // dbg!(&spec2);

    sm.state_manager
        .config_create(CreateConfig {
            id: Identity::new("my-id".to_string(), "my-name".to_string()),
            spec: spec,
        })
        .unwrap();
    sm.state_manager.reconciliation_loop().await;

    // println!("GreeterServer listening on {addr}");

    // Server::builder()
    //     .add_service(v1_svc::SystemManagerServiceServer::new(
    //         system_manager,
    //     ))
    //     .serve(addr)
    //     .await?;

    Ok(())
}
