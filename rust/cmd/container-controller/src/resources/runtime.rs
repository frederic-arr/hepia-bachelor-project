use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use bollard::Docker;
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    PrivateIdentity,
    Resource,
    ResourceResponse,
    Status,
    SubResourceCreate,
    ValidateResponse,
};
use cos_proto_state::v1::ReconcileNowRequest;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use system_controller::StaticFileSpec;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt as _, AsyncRead, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::STATE_CLIENT;

#[derive(Debug, Clone)]
pub struct RuntimeReconciler;

static ENGINES: LazyLock<Mutex<HashMap<String, Child>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub type RuntimeResource = Resource<RuntimeSpec, RuntimeDerivedSpec, DnsState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub engine: String,
    pub uid: u32,
    pub gid: u32,
    pub port: Option<u16>,
    pub depends_on: HashSet<Key>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeDerivedSpec {
    pub name: String,
    pub port: u16,
}

type DnsState = ();

impl RuntimeReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for RuntimeReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeReconciler {
    pub async fn validate(
        &self,
        key: Key,
        spec: RuntimeSpec,
        resource: Option<RuntimeResource>,
    ) -> Result<ValidateResponse<RuntimeDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(ValidateResponse {
            derived_spec: RuntimeDerivedSpec {
                name,
                port: spec
                    .port
                    .unwrap_or_else(|| rand::random_range(32768..60999)),
            },
            children: Self::get_children(&spec)?,
            dependencies: Self::get_deps(&spec),
        })
    }

    pub async fn reconcile(
        &self,
        resource: RuntimeResource,
    ) -> Result<ResourceResponse<Option<DnsState>>> {
        let k = &resource.derived_spec.name;
        let mut engines = ENGINES.lock().await;

        if matches!(resource.phase, Phase::Shutdown | Phase::Deleting) {
            if let Some(mut c) = engines.remove(k) {
                let _ = c.kill().await;
            }

            return Ok(ResourceResponse {
                status: Status::Deleted,
                state: None,
                children: vec![],
                dependencies: HashSet::new(),
            });
        }

        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: None,
                children: vec![],
                dependencies: Self::get_deps(&resource.spec),
            });
        }

        let children = Self::get_children(&resource.spec)?;
        if resource.children.len() != 1 {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children,
                dependencies: Self::get_deps(&resource.spec),
            });
        }

        for (existing, child) in resource.children.iter().zip(children.iter()) {
            if existing.id != child.id {
                return Ok(ResourceResponse {
                    status: Status::NotReady,
                    state: None,
                    children,
                    dependencies: Self::get_deps(&resource.spec),
                });
            }

            if existing.spec != child.spec {
                return Ok(ResourceResponse {
                    status: Status::NotReady,
                    state: None,
                    children,
                    dependencies: Self::get_deps(&resource.spec),
                });
            }

            if existing.status != Status::Done {
                return Ok(ResourceResponse {
                    status: Status::NotReady,
                    state: None,
                    children,
                    dependencies: Self::get_deps(&resource.spec),
                });
            }
        }

        if let Some(c) = engines.get_mut(k)
            && c.try_wait().is_ok_and(|o| o.is_none())
        {
            return Ok(ResourceResponse {
                status: Status::Ready,
                state: None,
                children,
                dependencies: resource
                    .spec
                    .depends_on
                    .iter()
                    .cloned()
                    .map(|v| Identity::Private(PrivateIdentity::Dynamic(v)))
                    .collect(),
            });
        }

        if let Some(mut c) = engines.remove(k) {
            let _ = c.kill().await;
        }

        engines.insert(k.to_owned(), Self::start_podman(&resource).await?);
        drop(engines);

        Ok(ResourceResponse {
            status: Status::NotReady,
            state: None,
            children,
            dependencies: resource
                .spec
                .depends_on
                .iter()
                .cloned()
                .map(|v| Identity::Private(PrivateIdentity::Dynamic(v)))
                .collect(),
        })
    }

    fn get_deps(spec: &RuntimeSpec) -> HashSet<Identity> {
        spec.depends_on
            .iter()
            .cloned()
            .map(|v| Identity::Private(PrivateIdentity::Dynamic(v)))
            .collect()
    }

    async fn start_podman(resource: &RuntimeResource) -> Result<Child> {
        let uid = resource.spec.uid;
        let gid = resource.spec.gid;
        let port_arg =
            format!("tcp://127.0.0.1:{}", resource.derived_spec.port);
        let home_dir = format!(
            "/var/lib/podman-data/{}",
            resource.derived_spec.name
        );
        std::fs::create_dir_all(&home_dir)?;
        std::os::unix::fs::chown(&home_dir, Some(uid), Some(gid))?;

        let mut binding = Command::new("/bin/podman");
        let cmd = binding
            .args([
                "--log-level=debug",
                "system",
                "service",
                "--time=0",
                &port_arg,
            ])
            .env("NETAVARK_FW", "nftables")
            .env("HOME", &home_dir)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .uid(uid)
            .gid(gid);

        // SAFETY: TODO
        unsafe {
            cmd.pre_exec(move || {
                std::fs::create_dir_all(format!("{home_dir}/.config"))?;

                Ok(())
            });
        }

        let mut child = cmd.spawn().context("unable to start podman")?;
        let file = tokio::fs::File::create("output.txt").await?;
        let err = tokio::fs::File::create("error.txt").await?;

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(forward_chunks(stdout, file));
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(forward_chunks(stderr, err));
        }

        let key = resource.id.key().clone();
        tokio::spawn(async move {
            let Ok(docker) = Docker::connect_with_host(&port_arg) else {
                return;
            };

            let docker = docker.with_timeout(Duration::from_secs(1));
            while docker.ping().await.is_err() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            let mut c = (*STATE_CLIENT).clone();
            let raw = match serde_json::to_vec(&key) {
                Ok(v) => v,
                Err(_err) => return,
            };

            let _ = timeout(
                Duration::from_secs(1),
                c.reconcile_now(ReconcileNowRequest { raw }),
            )
            .await;
        });

        Ok(child)
    }

    async fn validate_new_spec(&self, spec: &RuntimeSpec) -> Result<()> {
        if spec.engine != "podman" {
            bail!("only 'podman' is supported");
        }

        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &RuntimeResource,
        spec: &RuntimeSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_children(
        spec: &RuntimeSpec,
    ) -> Result<Vec<SubResourceCreate<Value>>> {
        Ok(vec![SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Dynamic(Key {
                schema: "system:static-file".to_owned(),
                name: Some("/etc/containers/policy.json".to_owned()),
            })),
            spec: serde_json::to_value(StaticFileSpec {
                path: "/etc/containers/policy.json".into(),
                content: Self::get_content(spec)?,
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: true,
            })?,
        }])
    }

    fn get_content(_spec: &RuntimeSpec) -> Result<String> {
        serde_json::to_string(&json!({
            "default": [
                {
                    "type": "insecureAcceptAnything"
                }
            ]
        }))
        .map_err(Into::into)
    }
}

#[expect(clippy::print_stdout, reason = "TODO")]
async fn forward_chunks<R>(stream: R, mut file: File) -> Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut reader = BufReader::new(stream).lines();

    while let Some(line) = reader.next_line().await? {
        for chunk in line.as_bytes().chunks(100) {
            let chunk = String::from_utf8_lossy(chunk);

            println!("{chunk}");
            file.write_all(chunk.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
    }

    Ok(())
}
