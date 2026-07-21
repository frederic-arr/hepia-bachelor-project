use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::LazyLock;

use anyhow::{Context as _, Result, bail};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    Resource,
    ResourceResponse,
    Status,
    SubResourceCreate,
    ValidateResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use system_controller::StaticFileSpec;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RuntimeReconciler;

static ENGINES: LazyLock<Mutex<HashMap<String, Child>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub type RuntimeResource = Resource<RuntimeSpec, DnsDerivedSpec, DnsState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub name: String,
    pub engine: String,
    pub uid: u32,
    pub gid: u32,
    pub depends_on: HashSet<Identity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DnsDerivedSpec {
    port: u16,
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
    const MAX_ATTEMPTS: u8 = 5;
    const MAX_NDOTS: u8 = 15;
    const MAX_NS: usize = 3;
    const MAX_SORTLIST: usize = 10;
    const MAX_TIMEOUT: u8 = 30;

    pub async fn validate(
        &self,
        spec: RuntimeSpec,
        resource: Option<RuntimeResource>,
    ) -> Result<ValidateResponse<DnsDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: DnsDerivedSpec {
                port: 42345,
                // port: rand::random_range(32768..60999),
            },
            children: vec![Self::get_child(&spec)?],
            dependencies: spec.depends_on,
        })
    }

    pub async fn reconcile(
        &self,
        resource: RuntimeResource,
    ) -> Result<ResourceResponse<Option<DnsState>>> {
        let k = &resource.spec.name;
        let mut engines = ENGINES.lock().await;

        if matches!(resource.phase, Phase::Shutdown | Phase::Teardown) {
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

        let child = Self::get_child(&resource.spec)?;
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: None,
                children: vec![],
                dependencies: resource.spec.depends_on,
            });
        }

        if resource.children.len() > 1 {
            return Ok(ResourceResponse {
                status: Status::Error("too many children".into()),
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        }

        let Some(existing) = resource.children.first() else {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        };

        if existing.id != child.id {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        }

        if existing.spec != child.spec {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        }

        if existing.status != Status::Done {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        }

        if let Some(c) = engines.get_mut(k)
            && c.try_wait().is_ok_and(|o| o.is_none())
        {
            return Ok(ResourceResponse {
                status: Status::Ready,
                state: None,
                children: vec![child],
                dependencies: resource.spec.depends_on,
            });
        }

        if let Some(mut c) = engines.remove(k) {
            let _ = c.kill().await;
        }

        engines.insert(k.to_owned(), Self::start_podman(&resource)?);
        drop(engines);

        Ok(ResourceResponse {
            status: Status::Ready,
            state: None,
            children: vec![child],
            dependencies: resource.spec.depends_on,
        })
    }

    fn start_podman(resource: &RuntimeResource) -> Result<Child> {
        let uid = resource.spec.uid;
        let gid = resource.spec.gid;
        let port_arg =
            format!("tcp://127.0.0.1:{}", resource.derived_spec.port);
        let home_dir = format!("/var/lib/podman-data/{}", resource.spec.name);
        std::fs::create_dir_all(&home_dir)?;
        std::os::unix::fs::chown(&home_dir, Some(uid), Some(gid))?;

        let mut binding = Command::new("/bin/podman");
        let cmd = binding
            .args(["system", "service", "--time=0", &port_arg])
            .env("NETAVARK_FW", "nftables")
            .env("HOME", &home_dir)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());

        // SAFETY: TODO
        unsafe {
            cmd.pre_exec(move || {
                if libc::setgroups(0, std::ptr::null()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }

                if libc::setresgid(gid, gid, gid) != 0 {
                    return Err(std::io::Error::last_os_error());
                }

                if libc::setresuid(uid, uid, uid) != 0 {
                    return Err(std::io::Error::last_os_error());
                }

                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }

                std::fs::create_dir_all(format!("{home_dir}/.config"))?;

                Ok(())
            });
        }

        cmd.spawn().context("unable to start podman")
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

    fn get_child(spec: &RuntimeSpec) -> Result<SubResourceCreate<Value>> {
        Ok(SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "system:static-file".to_owned(),
                name: Some("/etc/containers/policy.json".to_owned()),
            }),
            spec: serde_json::to_value(StaticFileSpec {
                path: "/etc/containers/policy.json".into(),
                content: Self::get_content(spec)?,
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: true,
            })?,
        })
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
