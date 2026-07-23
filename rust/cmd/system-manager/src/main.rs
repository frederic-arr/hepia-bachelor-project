#![feature(map_try_insert)]
#![feature(iterator_try_reduce)]
#![feature(iterator_try_collect)]

mod resources;
mod state_manager;

use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use cos_api_api::ApiResource;
use cos_api_api::proto::v1::{PushConfigRequest, PushConfigResponse};
use cos_api_api_server::proto::v1::{ApiService, ApiServiceServer};
use linux_utils::mount_iso;
use rustix::fs::sync;
use rustix::mount::{MountFlags, mount};
use rustix::system::RebootCommand;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::transport::Server;
use tonic::{Request, Response, Status, async_trait};
use tracing_subscriber::util::SubscriberInitExt;

use crate::resources::{
    DynamicResource,
    Identity,
    Resource,
    ResourceState,
    Spec,
    UserConfig,
};
use crate::state_manager::StateManager;

struct SystemManagerInner {
    state_manager: StateManager,
}

#[derive(Clone)]
pub struct SystemManagerService(Arc<RwLock<SystemManagerInner>>);

impl SystemManagerInner {
    fn new() -> Self {
        Self {
            state_manager: StateManager::new(),
        }
    }
}

impl SystemManagerService {
    fn new() -> Self {
        Self(Arc::new(RwLock::new(SystemManagerInner::new())))
    }

    async fn read(&self) -> RwLockReadGuard<'_, SystemManagerInner> {
        self.0.read().await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, SystemManagerInner> {
        self.0.write().await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallSpec {
    disk: String,
}

#[async_trait]
impl ApiService for SystemManagerService {
    async fn push_config(
        &self,
        request: Request<PushConfigRequest>,
    ) -> Result<Response<PushConfigResponse>, Status> {
        let resources: Vec<ApiResource> =
            rmp_serde::from_slice(request.into_inner().raw.as_slice()).unwrap();

        let mut w = self.write().await;
        w.state_manager.resources.retain(|k, v| {
            resources.iter().any(|r| {
                k == &Identity {
                    schema: r.schema.clone(),
                    name: r.name.clone(),
                }
            })
        });

        let is_installed = w
            .state_manager
            .resources
            .iter()
            .any(|(k, v)| k.schema == "containeros::system::install");

        let install_document = resources
            .iter()
            .find(|r| r.schema == "config#containeros::system::install");

        assert!(!is_installed || install_document.is_some());

        dbg!(is_maintenance(), is_installed, install_document);

        if is_maintenance()
            && !is_installed
            && let Some(doc) = install_document
        {
            tracing::warn!("installing to disk");
            let doc: InstallSpec = rmp_serde::from_slice(&doc.spec).unwrap();
            install(&doc.disk);
            std::fs::write("/config/config.json", "{}").unwrap();
            tracing::warn!("installed ok!");
        }

        for res in &resources {
            let id = Identity {
                schema: res.schema.clone(),
                name: res.name.clone(),
            };

            let result = w
                .state_manager
                .resources
                .entry(id)
                .and_modify(|r| {
                    let Resource::UserConfig(cfg) = r else {
                        panic!()
                    };

                    cfg.spec = Spec(res.spec.clone());
                })
                .or_insert_with(|| {
                    Resource::UserConfig(UserConfig {
                        schema: res.schema.clone(),
                        name: res.name.clone(),
                        spec: Spec(res.spec.clone()),
                        state: ResourceState::Unset,
                    })
                });
        }
        if std::fs::exists("/config/config.json").unwrap() {
            let ser = serde_json::to_string_pretty(
                &w.state_manager.resources.iter().collect::<Vec<(_, _)>>(),
            )
            .unwrap();
            std::fs::write("/config/config.json", ser).unwrap();
            sync();
        }

        if is_maintenance()
            && !is_installed
            && let Some(doc) = install_document
        {
            rustix::system::reboot(RebootCommand::Restart).unwrap();
        }
        drop(w);

        Ok(Response::new(PushConfigResponse::default()))
    }
}

fn install(disk: &str) {
    let mut child = Command::new("sgdisk").args(["-og", disk]).spawn().unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("sgdisk")
        .args(["-n", "0:0:+1MiB", "-t", "0:ef02", "-c", "0:boot", disk])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("sgdisk")
        .args(["-n", "0:0:+1GiB", "-t", "0:8300", "-c", "0:limine", disk])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("sgdisk")
        .args(["-n", "0:0:+1GiB", "-t", "0:8300", "-c", "0:config", disk])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("sgdisk")
        .args(["-n", "0:0:+1GiB", "-t", "0:8300", "-c", "0:data", disk])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("limine")
        .args(["bios-install", disk, "--force"])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "should be able to format disk");

    let mut child = Command::new("mkfs.vfat")
        .args(["-I", "-F32", &format!("{disk}2")])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "should be able to format filesystem"
    );

    let mut child = Command::new("mkfs.vfat")
        .args(["-I", "-F32", &format!("{disk}3")])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "should be able to format filesystem"
    );

    let mut child = Command::new("mkfs.ext4")
        .args(["-F", &format!("{disk}4")])
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "should be able to format filesystem"
    );

    std::fs::create_dir("/boot").unwrap();
    mount(
        "/dev/vda2",
        "/boot",
        "vfat",
        MountFlags::empty(),
        None,
    )
    .unwrap();

    std::fs::create_dir("/config").unwrap();
    mount(
        "/dev/vda3",
        "/config",
        "vfat",
        MountFlags::empty(),
        None,
    )
    .unwrap();

    std::fs::copy(
        "/share/limine/limine-bios.sys",
        "/boot/limine-bios.sys",
    )
    .unwrap();

    // TODO: mount partitions
    mount_iso("/mnt/iso", "/dev/sr0", MountFlags::empty(), &[]).unwrap();

    std::fs::copy("/mnt/iso/root.squashfs", "/boot/root.squashfs").unwrap();
    std::fs::copy("/mnt/iso/boot/bzImage", "/boot/bzImage").unwrap();
    std::fs::copy("/mnt/iso/boot/initrd", "/boot/initrd").unwrap();

    std::fs::write(
        "/boot/limine.conf",
        r"
timeout: 5

/ContainerOs
    protocol: linux
    path: boot():/bzImage
    cmdline: console=ttyS0,115200 init=/init
    module_path: boot():/initrd
    ",
    )
    .unwrap();

    sync();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut system_manager = SystemManagerService::new();
    let mut sm = system_manager.0.write().await;

    let spec = json!({
        "up": true,
        "ip_address": [10, 0, 2, 15],
        "ip_subnet": 24,
        "ip_gateway": [10, 0, 2, 2],
        "link_type": "Ethernet",
    });

    let spec = rmp_serde::to_vec(&spec).unwrap();
    let id = Identity {
        schema: "config#containeros::net::link".to_string(),
        name: "eth0".to_string(),
    };

    if is_maintenance() {
        sm.state_manager.resources.insert(
            id.clone(),
            resources::Resource::UserConfig(UserConfig {
                schema: id.schema,
                name: id.name,
                spec: Spec(spec),
                state: ResourceState::Unset,
            }),
        );
    } else {
        let ser = std::fs::read_to_string("/config/config.json").unwrap();
        let de: Vec<(Identity, Resource)> = serde_json::from_str(&ser).unwrap();
        sm.state_manager.resources = HashMap::from_iter(de);
    }

    drop(sm);

    // let spec = json!({
    // "image": "docker.io/library/busybox:latest",
    // "running": true,
    // "cmd": ["sleep", "infinity"]
    // });
    //
    // let spec = rmp_serde::to_vec(&spec).unwrap();
    // let id = Identity {
    // schema: "config#containeros::container::container".to_string(),
    // name: "bbox".to_string(),
    // };
    //
    // sm.state_manager.resources.insert(
    // id.clone(),
    // resources::Resource::UserConfig(UserConfig {
    // schema: id.schema,
    // name: id.name,
    // spec: Spec(spec),
    // state: ResourceState::Unset,
    // }),
    // );

    tokio::time::sleep(Duration::from_millis(500)).await;

    let sysm2 = system_manager.clone();
    tokio::task::spawn(async {
        tokio::time::sleep(Duration::from_millis(5000)).await;
        let addr = "0.0.0.0:50000".parse().unwrap();

        println!("api listening on {addr}");

        Server::builder()
            .add_service(ApiServiceServer::new(sysm2))
            .serve(addr)
            .await
            .unwrap();
    });
    system_manager.reconciliation_loop().await;
    Ok(())
}

#[must_use]
pub fn is_maintenance() -> bool {
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap();
    cmdline.contains("cos.maintenance")
}
