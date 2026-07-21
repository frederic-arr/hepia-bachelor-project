use std::collections::HashSet;

use anyhow::{Result, bail};
use cos_proto_reconciler::{
    Identity,
    Key,
    Resource,
    ResourceResponse,
    Status,
    SubResourceCreate,
    ValidateResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use system_controller::StaticFileSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReconciler;

pub type RuntimeResource = Resource<RuntimeSpec, DnsDerivedSpec, DnsState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub name: String,
    pub engine: String,
    pub uid: u32,
    pub gid: u32,
    pub depends_on: HashSet<Identity>,
}

type DnsDerivedSpec = ();
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
            derived_spec: (),
            children: vec![Self::get_child(&spec)?],
            dependencies: spec.depends_on,
        })
    }

    pub async fn reconcile(
        &self,
        resource: RuntimeResource,
    ) -> Result<ResourceResponse<Option<DnsState>>> {
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

        Ok(ResourceResponse {
            status: match existing.status {
                Status::Done => Status::Done,
                _ => Status::NotReady,
            },
            state: None,
            children: vec![child],
            dependencies: resource.spec.depends_on,
        })
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
