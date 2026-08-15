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
    ValidateResponse,
};
use cos_proto_state::v1::ReconcileNowRequest;
use serde::{Deserialize, Serialize};
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
    #[serde(default)]
    pub pools: Vec<String>,

    #[serde(default)]
    pub servers: Vec<String>,

    #[serde(default)]
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
            children: vec![],
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

        if let Some(c) = &mut *daemon
            && c.try_wait().is_ok_and(|o| o.is_some_and(|s| s.success()))
        {
            return Ok(ResourceResponse {
                status: Status::Done,
                state: None,
                children: vec![],
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

        *daemon = Some(Self::start_query(&resource)?);
        drop(daemon);

        Ok(ResourceResponse {
            status: Status::NotReady,
            state: None,
            children: vec![],
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

    fn start_query(resource: &NtpResource) -> Result<Child> {
        let mut binding = Command::new("/bin/ntpd");

        let servers = resource
            .spec
            .servers
            .iter()
            .flat_map(|v| ["-p".to_owned(), v.clone()]);

        let pools = resource
            .spec
            .pools
            .iter()
            .flat_map(|v| ["-p".to_owned(), v.clone()]);

        let cmd = binding
            .args(["-n", "-d", "-q"])
            .args(servers)
            .args(pools)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().context("unable to start ntp client")?;

        if let Some(mut stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let _ = wait_for_str(&mut stdout, "setting time to").await;
                let mut c = (*STATE_CLIENT).clone();

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
            });
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

pub async fn wait_for_str(out: &mut ChildStdout, pattern: &str) -> Result<()> {
    let reader = BufReader::new(out);
    let mut lines = reader.lines();
    loop {
        let ln = lines.next_line().await;
        let Some(line) = ln? else {
            bail!("reached end of line")
        };
        if line.contains(pattern) {
            return Ok(());
        }
    }
}
