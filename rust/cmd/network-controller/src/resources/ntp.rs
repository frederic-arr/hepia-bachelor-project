use std::collections::HashSet;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
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
use serde_json::Value;
use system_controller::StaticFileSpec;
use tokio::io::{AsyncBufReadExt as _, AsyncRead, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct NtpReconciler;

use super::STATE_CLIENT;

static DAEMON: LazyLock<Mutex<Option<Child>>> =
    LazyLock::new(|| Mutex::new(None));

pub type NtpResource = Resource<NtpSpec, NtpDerivedSpec, NtpState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NtpSpec {
    pub servers: Vec<String>,
    pub depends_on: HashSet<Key>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NtpDerivedSpec {}

type NtpState = ();

impl NtpReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NtpReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl NtpReconciler {
    pub async fn validate(
        &self,
        _key: Key,
        spec: NtpSpec,
        resource: Option<NtpResource>,
    ) -> Result<ValidateResponse<NtpDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: NtpDerivedSpec {},
            children: Self::get_children(&spec)?,
            dependencies: Self::get_deps(&spec),
        })
    }

    pub async fn reconcile(
        &self,
        resource: NtpResource,
    ) -> Result<ResourceResponse<Option<NtpState>>> {
        let mut daemon = DAEMON.lock().await;

        if matches!(resource.phase, Phase::Shutdown | Phase::Deleting) {
            if let Some(mut c) = daemon.take() {
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

        if let Some(c) = &mut *daemon
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

        if let Some(mut c) = daemon.take() {
            let _ = c.kill().await;
        }

        *daemon = Some(Self::start_daemon(&resource)?);
        drop(daemon);

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

    fn get_deps(spec: &NtpSpec) -> HashSet<Identity> {
        spec.depends_on
            .iter()
            .cloned()
            .map(|v| Identity::Private(PrivateIdentity::Dynamic(v)))
            .collect()
    }

    fn start_daemon(_resource: &NtpResource) -> Result<Child> {
        let mut binding = Command::new("/bin/ntpd");
        let cmd = binding
            .args(["-n"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().context("unable to start ntp daemon")?;

        if let Some(mut stdout) = child.stdout.take() {
            tokio::spawn(async move {
                tracing::warn!("waiting for time");
                let _ = wait_for_str(&mut stdout, "setting time to").await;
                let mut c = (*STATE_CLIENT).clone();
                tracing::warn!("got time");

                let Ok(raw) = serde_json::to_vec(&Key {
                    schema: "network:ntp".to_owned(),
                    name: None,
                }) else {
                    return;
                };

                let _ = timeout(
                    Duration::from_secs(1),
                    c.reconcile_now(ReconcileNowRequest { raw }),
                )
                .await;

                let _ = forward_chunks(stdout).await;
            });
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(forward_chunks(stderr));
        }

        Ok(child)
    }

    async fn validate_new_spec(&self, _spec: &NtpSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &NtpResource,
        spec: &NtpSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_children(spec: &NtpSpec) -> Result<Vec<SubResourceCreate<Value>>> {
        Ok(vec![SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Dynamic(Key {
                schema: "system:static-file".to_owned(),
                name: Some("/etc/ntp.conf".to_owned()),
            })),
            spec: serde_json::to_value(StaticFileSpec {
                path: "/etc/ntp.conf".into(),
                content: spec
                    .servers
                    .iter()
                    .map(|v| format!("server {v} iburst"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: true,
            })?,
        }])
    }
}

#[expect(clippy::print_stdout, reason = "TODO")]
async fn forward_chunks<R>(stream: R) -> Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut reader = BufReader::new(stream).lines();
    while let Some(line) = reader.next_line().await? {
        for chunk in line.as_bytes().chunks(100) {
            println!("{}", String::from_utf8_lossy(chunk));
        }
    }
    Ok(())
}

#[expect(clippy::print_stdout, reason = "TODO")]
pub async fn wait_for_str(out: &mut ChildStdout, pattern: &str) -> Result<()> {
    let reader = BufReader::new(out);
    let mut lines = reader.lines();
    loop {
        let ln = lines.next_line().await;
        let Some(line) = ln? else {
            bail!("reached end of line")
        };
        println!("{line}");
        if line.contains(pattern) {
            return Ok(());
        }
    }
}
