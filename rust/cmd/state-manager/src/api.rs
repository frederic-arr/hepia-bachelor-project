use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use argon2::{Argon2, PasswordHash, PasswordVerifier as _};
use cos_proto_api::v1::{
    ConfigPullRequest,
    ConfigPullResponse,
    ConfigPushRequest,
    ConfigPushResponse,
    ConfigValidateRequest,
    ConfigValidateResponse,
    FsListRequest,
    FsListResponse,
    FsReadRequest,
    FsReadResponse,
    FsWriteRequest,
    FsWriteResponse,
    ResourcesForceDeleteRequest,
    ResourcesForceDeleteResponse,
    ResourcesGetRequest,
    ResourcesGetResponse,
    ResourcesListRequest,
    ResourcesListResponse,
    ResourcesReconcileNowRequest,
    ResourcesReconcileNowResponse,
    SystemRebootRequest,
    SystemRebootResponse,
};
use cos_proto_api::{
    ConfigPullResponsePayload,
    ConfigValidateRequestPayload,
    ConfigValidateResponsePayload,
    FsListRequestPayload,
    FsListResponsePayload,
    FsReadRequestPayload,
    FsReadResponsePayload,
};
use cos_proto_api_server::v1::ApiService;
use cos_proto_reconciler::{Identity, Key, PrivateIdentity, SubResourceCreate};
use itertools::Itertools as _;
use linux_utils::{is_maintenance, mount_iso};
use rustix::fs::sync;
use rustix::mount::{MountFlags, mount};
use rustix::system::{RebootCommand, reboot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::state::StateManager;

pub struct ApiServer {
    pub sm: Arc<StateManager>,
    pub config: Mutex<ApiConfig>,
    pub cmdline: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub auth: ApiAuth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallConfig {
    pub system_disk: String,
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

    async fn install(&self, config: &InstallConfig) -> Result<()> {
        if !is_maintenance() {
            bail!("cannot install outside of maintenance mode");
        }

        let disk = &config.system_disk;
        let mut child = Command::new("sgdisk").args(["-og", disk]).spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+1MiB", "-t", "0:ef02", "-c", "0:boot", disk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+512MiB", "-t", "0:8300", "-c", "0:limine", disk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+10MiB", "-t", "0:8300", "-c", "0:config", disk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:0", "-t", "0:8300", "-c", "0:data", disk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("limine")
            .args(["bios-install", disk, "--force"])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let bootdisk = format!("{disk}2");
        let mut child = Command::new("mkfs.vfat")
            .args(["-I", "-F32", &bootdisk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let configdisk = format!("{disk}3");
        let mut child = Command::new("mkfs.vfat")
            .args(["-I", "-F32", &configdisk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let datadisk = format!("{disk}4");
        let mut child =
            Command::new("mkfs.ext4").args(["-F", &datadisk]).spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        tracing::warn!("mounting partitions");
        std::fs::create_dir_all("/boot")?;
        mount(
            &bootdisk,
            "/boot",
            "vfat",
            MountFlags::empty(),
            None,
        )?;

        std::fs::create_dir_all("/config")?;
        mount(
            &configdisk,
            "/config",
            "vfat",
            MountFlags::empty(),
            None,
        )?;

        tracing::warn!("copying limine files");
        std::fs::copy(
            "/share/limine/limine-bios.sys",
            "/boot/limine-bios.sys",
        )?;

        mount_iso("/mnt/iso", "/dev/sr0", MountFlags::empty(), &[])?;

        tracing::warn!("copying boot assets");
        std::fs::copy("/mnt/iso/root.squashfs", "/boot/root.squashfs")?;
        std::fs::copy("/mnt/iso/boot/bzImage", "/boot/bzImage")?;
        std::fs::copy("/mnt/iso/boot/initrd", "/boot/initrd")?;

        tracing::warn!("creating boot config");
        std::fs::write(
            "/boot/limine.conf",
            format!(
                "
timeout: 5

/ContainerOs
    protocol: linux
    path: boot():/bzImage
    cmdline: console=ttyS0,115200 init=/init cos.bootdisk={bootdisk} \
                 cos.configdisk={configdisk} cos.datadisk={datadisk}
    module_path: boot():/initrd

/ContainerOs (maintenance)
    protocol: linux
    path: boot():/bzImage
    cmdline: console=ttyS0,115200 init=/init cos.maintenance
    module_path: boot():/initrd
    "
            ),
        )?;

        sync();

        Ok(())
    }
}

#[tonic::async_trait]
impl ApiService for ApiServer {
    async fn config_validate(
        &self,
        request: Request<ConfigValidateRequest>,
    ) -> Result<Response<ConfigValidateResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let payload: ConfigValidateRequestPayload =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        let clients = self.sm.clients.read().await;
        let resources = self.sm.resources.read().await;
        let validation = StateManager::bulk_validate(
            &clients,
            &resources,
            payload.resources,
        )
        .await;
        drop(clients);

        let response = match validation {
            Ok(_) => ConfigValidateResponsePayload::Ok,
            Err(err) => ConfigValidateResponsePayload::Error(err.to_string()),
        };

        Ok(Response::new(ConfigValidateResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    #[expect(
        clippy::significant_drop_tightening,
        reason = "the guard is leaked on purpose"
    )]
    async fn config_push(
        &self,
        request: Request<ConfigPushRequest>,
    ) -> Result<Response<ConfigPushResponse>, Status> {
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

        let install_cfg = resources
            .iter()
            .find(|v| {
                v.id.key()
                    == &Key {
                        schema: "install".to_owned(),
                        name: None,
                    }
            })
            .cloned();

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
                Identity::Private(
                    PrivateIdentity::Dynamic(_) | PrivateIdentity::Ephemeral(_),
                )
                | Identity::Shared(_) => None,
            })
            .cloned()
            .collect();

        let updated_resources = resources
            .into_iter()
            .map(|v| (v.id.key().clone(), v))
            .collect();

        let should_schedule = !is_maintenance() || install_cfg.is_none();
        StateManager::bulk_upsert(
            &clients,
            &self.sm.queue,
            &mut guard,
            old_keys,
            updated_resources,
            should_schedule,
        )
        .await
        .map_err(|err| Status::from_error(err.into()))?;
        drop(guard);

        if is_maintenance()
            && let Some(cfg) = install_cfg
        {
            let guard = self.sm.queue.block().await;
            tracing::warn!("installing...");
            let cfg: InstallConfig = serde_json::from_value(cfg.spec)
                .map_err(|err| Status::from_error(err.into()))?;

            self.install(&cfg)
                .await
                .map_err(|err| Status::from_error(err.into()))?;

            self.sm
                .serialize_bundle()
                .await
                .map_err(|err| Status::from_error(err.into()))?;

            std::mem::forget(guard);
            std::thread::spawn(|| {
                tracing::info!("install succesfull, rebooting in 3 seconds");
                std::thread::sleep(Duration::from_secs(3));

                let _ = reboot(RebootCommand::Restart);
            });
        }

        Ok(Response::new(ConfigPushResponse { raw: vec![] }))
    }

    async fn config_pull(
        &self,
        request: Request<ConfigPullRequest>,
    ) -> Result<Response<ConfigPullResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let resources = self
            .sm
            .resources
            .read()
            .await
            .values()
            .filter_map(|v| {
                if !matches!(
                    v.id,
                    Identity::Private(PrivateIdentity::Static(_))
                ) {
                    return None;
                }

                Some(SubResourceCreate {
                    id: v.id.clone(),
                    spec: v.spec.clone(),
                })
            })
            .collect_vec();

        let response = ConfigPullResponsePayload { resources };

        Ok(Response::new(ConfigPullResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn fs_write(
        &self,
        request: Request<FsWriteRequest>,
    ) -> Result<Response<FsWriteResponse>, Status> {
        self.auth_or_fail(&request).await?;
        todo!()
    }

    async fn fs_list(
        &self,
        request: Request<FsListRequest>,
    ) -> Result<Response<FsListResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let payload: FsListRequestPayload = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        let entries: Vec<_> = std::fs::read_dir(payload.path)?.try_collect()?;

        let response = FsListResponsePayload {
            entries: entries
                .into_iter()
                .map(|v| {
                    Ok::<_, anyhow::Error>((
                        v.file_type()?.is_dir(),
                        v.file_name().to_string_lossy().to_string(),
                    ))
                })
                .try_collect()
                .map_err(|err| Status::from_error(err.into()))?,
        };

        Ok(Response::new(FsListResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn fs_read(
        &self,
        request: Request<FsReadRequest>,
    ) -> Result<Response<FsReadResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let payload: FsReadRequestPayload = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        let data = std::fs::read(payload.path)?;
        let response = FsReadResponsePayload { content: data };

        Ok(Response::new(FsReadResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn resources_list(
        &self,
        request: Request<ResourcesListRequest>,
    ) -> Result<Response<ResourcesListResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let guard = self.sm.resources.read().await;
        let resources = guard.values().cloned().collect_vec();
        drop(guard);

        Ok(Response::new(ResourcesListResponse {
            raw: serde_json::to_vec(&resources)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn resources_get(
        &self,
        request: Request<ResourcesGetRequest>,
    ) -> Result<Response<ResourcesGetResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;
        let guard = self.sm.resources.read().await;
        let resources = guard.get(&key).cloned();
        drop(guard);

        Ok(Response::new(ResourcesGetResponse {
            raw: serde_json::to_vec(&resources)
                .map_err(|err| Status::from_error(err.into()))?,
        }))
    }

    async fn resources_reconcile_now(
        &self,
        request: Request<ResourcesReconcileNowRequest>,
    ) -> Result<Response<ResourcesReconcileNowResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();

        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;
        self.sm.queue.schedule_at(key, Instant::now()).await;

        Ok(Response::new(ResourcesReconcileNowResponse {
            raw: vec![],
        }))
    }

    async fn resources_force_delete(
        &self,
        request: Request<ResourcesForceDeleteRequest>,
    ) -> Result<Response<ResourcesForceDeleteResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        tracing::info!(%key, "forced deletion");

        let mut guard = self.sm.resources.write().await;
        let _ = guard.remove(&key);
        drop(guard);

        Ok(Response::new(ResourcesForceDeleteResponse {
            raw: vec![],
        }))
    }

    async fn system_reboot(
        &self,
        request: Request<SystemRebootRequest>,
    ) -> Result<Response<SystemRebootResponse>, Status> {
        self.auth_or_fail(&request).await?;
        todo!()
    }
}
