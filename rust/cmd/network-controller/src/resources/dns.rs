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
use serde_json::Value;
use system_controller::StaticFileSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsReconciler;

pub type DnsResource = Resource<DnsSpec, DnsDerivedSpec, DnsState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Resolver configuration (see [`resolv.conf(5)`](https://www.man7.org/linux/man-pages/man5/resolv.conf.5.html)).
pub struct DnsSpec {
    pub nameservers: Vec<String>,
    pub search: Option<Vec<String>>,
    pub sortlist: Option<Vec<String>>,
    pub ndots: Option<u8>,
    pub timeout: Option<u8>,
    pub attempts: Option<u8>,
    pub debug: Option<bool>,
    pub rotate: Option<bool>,
    pub no_aaaa: Option<bool>,
    pub no_check_names: Option<bool>,
    pub inet6: Option<bool>,
    pub ip6_bytestring: Option<bool>,
    pub ip6_dotint: Option<bool>,
    pub ip6_no_dotint: Option<bool>,
    pub edns0: Option<bool>,
    pub single_request: Option<bool>,
    pub single_request_reopen: Option<bool>,
    pub no_tld_query: Option<bool>,
    pub use_vc: Option<bool>,
    pub no_reload: Option<bool>,
    pub trust_ad: Option<bool>,
}

type DnsDerivedSpec = ();
type DnsState = ();

impl DnsReconciler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for DnsReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsReconciler {
    const MAX_ATTEMPTS: u8 = 5;
    const MAX_NDOTS: u8 = 15;
    const MAX_NS: usize = 3;
    const MAX_SORTLIST: usize = 10;
    const MAX_TIMEOUT: u8 = 30;

    pub async fn validate(
        &self,
        spec: DnsSpec,
        resource: Option<DnsResource>,
    ) -> Result<ValidateResponse<DnsDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: (),
            children: vec![Self::get_child(&spec)?],
            dependencies: vec![],
        })
    }

    pub async fn reconcile(
        &self,
        resource: DnsResource,
    ) -> Result<ResourceResponse<Option<DnsState>>> {
        let child = Self::get_child(&resource.spec)?;
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}").into()),
                state: None,
                children: vec![],
                dependencies: HashSet::new(),
            });
        }

        if resource.children.len() > 1 {
            return Ok(ResourceResponse {
                status: Status::Error("too many children".into()),
                state: None,
                children: vec![child],
                dependencies: HashSet::new(),
            });
        }

        let Some(existing) = resource.children.first() else {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: HashSet::new(),
            });
        };

        if existing.id != child.id {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: HashSet::new(),
            });
        }

        if existing.spec != child.spec {
            return Ok(ResourceResponse {
                status: Status::NotReady,
                state: None,
                children: vec![child],
                dependencies: HashSet::new(),
            });
        }

        Ok(ResourceResponse {
            status: match existing.status {
                Status::Done => Status::Done,
                _ => Status::NotReady,
            },
            state: None,
            children: vec![child],
            dependencies: HashSet::new(),
        })
    }

    async fn validate_new_spec(&self, spec: &DnsSpec) -> Result<()> {
        if spec.nameservers.len() > Self::MAX_NS {
            bail!(
                "a maximum of {} name servers can be specified",
                Self::MAX_NS
            );
        }

        if spec
            .sortlist
            .as_ref()
            .is_some_and(|v| v.len() > Self::MAX_SORTLIST)
        {
            bail!(
                "a maximum of {} sortlist paris can be specified",
                Self::MAX_SORTLIST
            );
        }

        if spec.ndots.is_some_and(|v| v > Self::MAX_NDOTS) {
            bail!("ndots cannot be greater than {}", Self::MAX_NDOTS);
        }

        if spec.timeout.is_some_and(|v| v > Self::MAX_TIMEOUT) {
            bail!(
                "timeout cannot be greater than {}",
                Self::MAX_TIMEOUT
            );
        }

        if spec.attempts.is_some_and(|v| v > Self::MAX_ATTEMPTS) {
            bail!(
                "attempts cannot be greater than {}",
                Self::MAX_ATTEMPTS
            );
        }

        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &DnsResource,
        spec: &DnsSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    fn get_child(spec: &DnsSpec) -> Result<SubResourceCreate<Value>> {
        Ok(SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "system:static-file".to_owned(),
                name: Some("/etc/resolv.conf".to_owned()),
            }),
            spec: serde_json::to_value(StaticFileSpec {
                path: "/etc/resolv.conf".into(),
                content: Self::get_content(spec),
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: true,
            })?,
        })
    }

    fn get_content(spec: &DnsSpec) -> String {
        fn opt(s: &str) -> impl FnOnce(bool) -> Option<String> {
            let s = s.to_owned();
            move |b| b.then_some(s)
        }

        let options = vec![
            spec.ndots.map(|n| format!("ndots:{n}")),
            spec.timeout.map(|n| format!("timeout:{n}")),
            spec.attempts.map(|n| format!("attempts:{n}")),
            spec.debug.and_then(opt("debug")),
            spec.rotate.and_then(opt("rotate")),
            spec.no_aaaa.and_then(opt("no-aaaa")),
            spec.no_check_names.and_then(opt("no-check-names")),
            spec.inet6.and_then(opt("inet6")),
            spec.ip6_bytestring.and_then(opt("ip6-bytestring")),
            spec.ip6_dotint.and_then(opt("ip6-dotint")),
            spec.ip6_no_dotint.and_then(opt("ip6-no-dotint")),
            spec.edns0.and_then(opt("edns0")),
            spec.single_request.and_then(opt("single-request")),
            spec.single_request_reopen
                .and_then(opt("single-request-reopen")),
            spec.no_tld_query.and_then(opt("no-tld-query")),
            spec.use_vc.and_then(opt("use-vc")),
            spec.no_reload.and_then(opt("no-reload")),
            spec.trust_ad.and_then(opt("trust-ad")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");

        let mut lines = Vec::with_capacity(Self::MAX_NS + 3);
        if !options.is_empty() {
            lines.push(format!("options {options}"));
        }

        if let Some(search) = &spec.search {
            let search = search.join(" ");
            if !search.is_empty() {
                lines.push(format!("search {search}"));
            }
        }

        if let Some(sortlist) = &spec.sortlist {
            let sortlist = sortlist.join(" ");
            if !sortlist.is_empty() {
                lines.push(format!("sortlist {sortlist}"));
            }
        }

        for ns in &spec.nameservers {
            lines.push(format!("nameserver {ns}"));
        }

        lines.push(String::new());
        lines.join("\n")
    }
}
