#![feature(map_try_insert)]
#![feature(iterator_try_reduce)]
#![feature(iterator_try_collect)]

mod resources;
mod state_manager;

use std::collections::HashSet;
use std::time::Duration;

use serde_json::json;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::{Request, Response, Status};
use tracing_subscriber::util::SubscriberInitExt;

use crate::resources::{
    DynamicResource,
    Identity,
    ResourceState,
    Spec,
    UserConfig,
};
use crate::state_manager::StateManager;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let system_manager = SystemManagerService::new();
    let mut sm = system_manager.0.write().await;

    let spec = json!({
        "up": true,
    });

    let spec = rmp_serde::to_vec(&spec).unwrap();
    let id = Identity {
        schema: "config#containeros::net::link".to_string(),
        name: "dummy0".to_string(),
    };

    // sm.state_manager.resources.insert(
    //     id.clone(),
    //     resources::Resource::UserConfig(UserConfig {
    //         schema: id.schema,
    //         name: id.name,
    //         spec: Spec(spec),
    //         state: ResourceState::Unset,
    //     }),
    // );

    let spec = json!({
        "image": "docker.io/library/busybox:latest",
        "running": true,
        "cmd": ["sleep", "infinity"]
    });

    let spec = rmp_serde::to_vec(&spec).unwrap();
    let id = Identity {
        schema: "config#containeros::container::container".to_string(),
        name: "bbox3".to_string(),
    };

    sm.state_manager.resources.insert(
        id.clone(),
        resources::Resource::UserConfig(UserConfig {
            schema: id.schema,
            name: id.name,
            spec: Spec(spec),
            state: ResourceState::Unset,
        }),
    );

    tokio::time::sleep(Duration::from_millis(500)).await;
    // sm.state_manager.reconciliation_loop().await;
    drop(sm);
    Ok(())
}
