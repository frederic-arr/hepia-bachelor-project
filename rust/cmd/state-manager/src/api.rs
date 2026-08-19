use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use argon2::{Argon2, PasswordHash, PasswordVerifier as _};
use cos_proto_api::v1::{
    ConfigPullRequest,
    ConfigPullResponse,
    ConfigPushRequest,
    ConfigPushResponse,
    ConfigPushStrRequest,
    ConfigPushStrResponse,
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
    ConfigResource,
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
use rustix::thread::{CapabilitySet, CapabilitySets, set_capabilities};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::CAPS;
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
    pub disks: Vec<InstallDisk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallDisk {
    dev: String,
    #[serde(default)]
    partitions: Vec<DiskPartition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum DiskPartition {
    Boot,
    Config {
        #[serde(default)]
        encryption: Option<DiskEncryption>,
    },
    Data {
        #[serde(default)]
        encryption: Option<DiskEncryption>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "provider")]
pub enum DiskEncryption {
    Static { key: String, autounlock: bool },
    Tpm2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiAuth {
    None,
    Password(String),
}

type MaybeDisk<'cfg> = Option<(&'cfg String, &'cfg DiskPartition)>;

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

    async fn install(
        &self,
        old_config: Option<&InstallConfig>,
        new_config: &InstallConfig,
    ) -> Result<bool> {
        if !is_maintenance() {
            bail!("cannot install outside of maintenance mode");
        }

        set_capabilities(
            None,
            CapabilitySets {
                effective: CAPS,
                permitted: CAPS,
                inheritable: CapabilitySet::empty(),
            },
        )
        .map_err(|err| Status::from_error(err.into()))?;

        Self::install_disks(old_config, new_config)
    }

    fn get_config_layout(
        config: &InstallConfig,
    ) -> Result<(MaybeDisk<'_>, MaybeDisk<'_>, MaybeDisk<'_>)> {
        let boot_disk = config
            .disks
            .iter()
            .flat_map(|v| {
                v.partitions
                    .iter()
                    .filter(|v| matches!(v, DiskPartition::Boot))
                    .map(|vv| (&v.dev, vv))
                    .collect_vec()
            })
            .collect_vec();

        let config_disk = config
            .disks
            .iter()
            .flat_map(|v| {
                v.partitions
                    .iter()
                    .filter(|v| matches!(v, DiskPartition::Config { .. }))
                    .map(|vv| (&v.dev, vv))
                    .collect_vec()
            })
            .collect_vec();

        let data_disk = config
            .disks
            .iter()
            .flat_map(|v| {
                v.partitions
                    .iter()
                    .filter(|v| matches!(v, DiskPartition::Data { .. }))
                    .map(|vv| (&v.dev, vv))
                    .collect_vec()
            })
            .collect_vec();

        if boot_disk.len() > 1 {
            bail!("can only specify one boot partition");
        }

        if config_disk.len() > 1 {
            bail!("can only specify one config partition");
        }

        if data_disk.len() > 1 {
            bail!("can only specify one data partition");
        }

        let boot_disk = boot_disk.first().copied();
        let config_disk = config_disk.first().copied();
        let data_disk = data_disk.first().copied();

        Ok((boot_disk, config_disk, data_disk))
    }

    fn install_disks(
        old_config: Option<&InstallConfig>,
        new_config: &InstallConfig,
    ) -> Result<bool> {
        let mut should_reboot = true;
        let (old_boot_disk, old_config_disk, old_data_disk) = old_config
            .map(Self::get_config_layout)
            .transpose()?
            .unwrap_or_default();

        let (new_boot_disk, new_config_disk, new_data_disk) =
            Self::get_config_layout(new_config)?;

        let boot_disk = match (old_boot_disk, new_boot_disk) {
            (Some((old, _)), Some((new, _))) => {
                if old != new {
                    bail!("cannot move boot disk to another partition")
                }

                None
            }
            (Some(_), None) => bail!("cannot remove boot disk"),
            (None, Some((dev, _))) => {
                if let Some((existing_config_dev, _)) = old_config_disk
                    && existing_config_dev == dev
                {
                    bail!("cannot append partition after config");
                }

                if let Some((existing_data_dev, _)) = old_data_disk
                    && existing_data_dev == dev
                {
                    bail!("cannot append partition after data");
                }

                Some(Self::partition_boot_disk(dev)?)
            }
            (None, None) => None,
        };

        let config_disk = match (old_config_disk, new_config_disk) {
            (Some((old, _)), Some((new, _))) => {
                if old != new {
                    bail!("cannot move boot disk to another partition")
                }

                None
            }
            (Some(_), None) => bail!("cannot remove boot disk"),
            (None, Some((dev, _))) => {
                if let Some((existing_data_dev, _)) = old_data_disk
                    && existing_data_dev == dev
                {
                    bail!("cannot append partition after data");
                }

                Some(Self::partition_config_disk(
                    dev,
                    boot_disk.as_ref().is_some_and(|v| v.starts_with(dev)),
                )?)
            }
            (None, None) => None,
        };

        let data_disk = match (old_data_disk, new_data_disk) {
            (Some((old, _)), Some((new, _))) => {
                if old != new {
                    bail!("cannot move boot disk to another partition")
                }

                None
            }
            (Some(_), None) => bail!("cannot remove boot disk"),
            (None, Some((dev, _))) => {
                should_reboot = true;
                Some(Self::partition_data_disk(
                    dev,
                    boot_disk.as_ref().is_some_and(|v| v.starts_with(dev)),
                    config_disk.as_ref().is_some_and(|v| v.starts_with(dev)),
                )?)
            }
            (None, None) => None,
        };

        if let Some(boot_disk) = &boot_disk {
            Self::install_boot_disk(boot_disk)?;
        }

        if let Some(config_disk) = &config_disk {
            Self::install_config_disk(config_disk)?;
        }

        if let Some(data_disk) = &data_disk {
            Self::install_data_disk(data_disk)?;
        }

        if let Some(boot_disk) = boot_disk {
            let boot_disk = format!("cos.bootdisk={boot_disk}");

            let config_disk =
                config_disk.map_or_default(|v| format!("cos.configdisk={v}"));

            let data_disk =
                data_disk.map_or_default(|v| format!("cos.datadisk={v}"));

            std::fs::write(
                "/boot/limine.conf",
                format!(
                    "
timeout: 5

/ContainerOs
    protocol: linux
    path: boot():/bzImage
    cmdline: console=ttyS0,115200 init=/init {boot_disk} {config_disk} \
                     {data_disk}
    module_path: boot():/initrd

/ContainerOs (maintenance)
    protocol: linux
    path: boot():/bzImage
    cmdline: console=ttyS0,115200 init=/init cos.maintenance
    module_path: boot():/initrd
    "
                ),
            )?;
        }

        sync();

        Ok(should_reboot)
    }

    fn partition_boot_disk(dev: &str) -> Result<String> {
        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+1MiB", "-t", "0:ef02", "-c", "0:boot", dev])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+512MiB", "-t", "0:8300", "-c", "0:limine", dev])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let mut child = Command::new("limine")
            .args(["bios-install", dev, "--force"])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let bootdisk = format!("{dev}2");
        Ok(bootdisk)
    }

    fn install_boot_disk(bootdisk: &str) -> Result<()> {
        let mut child = Command::new("mkfs.vfat")
            .args(["-I", "-F32", bootdisk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        std::fs::create_dir_all("/boot")?;
        mount(
            bootdisk,
            "/boot",
            "vfat",
            MountFlags::empty(),
            None,
        )?;

        std::fs::copy(
            "/share/limine/limine-bios.sys",
            "/boot/limine-bios.sys",
        )?;

        mount_iso("/mnt/iso", "/dev/sr0", MountFlags::empty(), &[])?;

        std::fs::copy("/mnt/iso/root.squashfs", "/boot/root.squashfs")?;
        std::fs::copy("/mnt/iso/boot/bzImage", "/boot/bzImage")?;
        std::fs::copy("/mnt/iso/boot/initrd", "/boot/initrd")?;

        Ok(())
    }

    fn partition_config_disk(dev: &str, has_boot: bool) -> Result<String> {
        dbg!(&dev, &has_boot);
        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:+10MiB", "-t", "0:8300", "-c", "0:config", dev])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let configdisk = format!("{dev}{}", if has_boot { 3 } else { 1 });
        Ok(configdisk)
    }

    fn install_config_disk(configdisk: &str) -> Result<()> {
        let mut child = Command::new("mkfs.vfat")
            .args(["-I", "-F32", configdisk])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        std::fs::create_dir_all("/config")?;
        mount(
            configdisk,
            "/config",
            "vfat",
            MountFlags::empty(),
            None,
        )?;

        Ok(())
    }

    #[expect(clippy::arithmetic_side_effects, reason = "Using constant values")]
    fn partition_data_disk(
        dev: &str,
        has_boot: bool,
        has_config: bool,
    ) -> Result<String> {
        let mut child = Command::new("sgdisk")
            .args(["-n", "0:0:0", "-t", "0:8300", "-c", "0:data", dev])
            .spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        let datadisk = format!(
            "{dev}{}",
            if has_boot { 3 } else { 1 } + i32::from(has_config)
        );

        Ok(datadisk)
    }

    fn install_data_disk(datadisk: &str) -> Result<()> {
        let mut child =
            Command::new("mkfs.ext4").args(["-F", datadisk]).spawn()?;
        let status = child.wait()?;
        status.exit_ok()?;

        Ok(())
    }

    #[expect(
        clippy::significant_drop_tightening,
        reason = "the guard is leaked on purpose"
    )]
    async fn config_push_inner(
        &self,
        resources: Vec<SubResourceCreate<Value>>,
    ) -> Result<(), Status> {
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

        let mut guard = self.sm.resources.write().await;
        let old_install = guard
            .values()
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

        let should_reboot = match (old_install, install_cfg) {
            (None, None) => false,
            (Some(_), None) => {
                return Err(Status::from_error(
                    anyhow!("cannot remove install config").into(),
                ));
            }
            (old, Some(new)) => {
                if old.as_ref().is_none_or(|v| v.spec != new.spec) {
                    let old: Option<InstallConfig> = old
                        .map(|v| serde_json::from_value(v.spec))
                        .transpose()
                        .map_err(|err| Status::from_error(err.into()))?;

                    let new: InstallConfig =
                        serde_json::from_value(new.spec)
                            .map_err(|err| Status::from_error(err.into()))?;

                    self.install(old.as_ref(), &new)
                        .await
                        .map_err(|err| Status::from_error(err.into()))?
                } else {
                    false
                }
            }
        };

        StateManager::bulk_upsert(
            &clients,
            &self.sm.queue,
            &mut guard,
            old_keys,
            updated_resources,
            !should_reboot,
        )
        .await
        .map_err(|err| Status::from_error(err.into()))?;
        drop(guard);

        self.sm
            .serialize_bundle()
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        if should_reboot {
            let guard = self.sm.queue.block().await;
            std::mem::forget(guard);
            std::thread::spawn(|| {
                tracing::info!("install succesfull, rebooting in 3 seconds");
                std::thread::sleep(Duration::from_secs(3));

                let _ = reboot(RebootCommand::Restart);
            });
        }

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

    async fn config_push(
        &self,
        request: Request<ConfigPushRequest>,
    ) -> Result<Response<ConfigPushResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let resources: Vec<SubResourceCreate<Value>> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        self.config_push_inner(resources).await?;
        Ok(Response::new(ConfigPushResponse { raw: vec![] }))
    }

    async fn config_push_str(
        &self,
        request: Request<ConfigPushStrRequest>,
    ) -> Result<Response<ConfigPushStrResponse>, Status> {
        self.auth_or_fail(&request).await?;
        let req = request.into_inner();
        let configs = serde_yaml::Deserializer::from_str(&req.yaml)
            .map(ConfigResource::deserialize)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| Status::from_error(err.into()))?
            .into_iter()
            .map(|v| SubResourceCreate {
                id: Identity::Private(PrivateIdentity::Static(Key {
                    schema: v.schema,
                    name: v.name,
                })),
                spec: v.spec,
            })
            .collect::<Vec<_>>();

        self.config_push_inner(configs).await?;
        Ok(Response::new(ConfigPushStrResponse {
            raw: vec![],
        }))
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
