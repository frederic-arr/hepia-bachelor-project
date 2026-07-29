use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::os::fd::{AsRawFd as _, FromRawFd as _, OwnedFd};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
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
use cos_proto_state_client::v1::StateServiceClient;
use rtnetlink::Handle;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smoltcp::iface::{Config, Interface, PollIngressSingleResult, SocketSet};
use smoltcp::phy::{Medium, RawSocket};
use smoltcp::socket::dhcpv4;
use smoltcp::wire::EthernetAddress;
use tokio::io::unix::AsyncFd;
use tokio::sync::Mutex;
use tokio::task::yield_now;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Channel, Endpoint};

use crate::{AddressSpec, LinkContext, LinkReconciler, RouteSpec};

#[derive(Debug, Clone)]
pub struct DhcpReconciler {
    rtnl: Handle,
}

pub type DhcpResource = Resource<DhcpSpec, DhcpDerivedSpec, DhcpState>;

static CLIENTS: LazyLock<Mutex<HashMap<String, DhcpWorkState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static STATE_CLIENT: LazyLock<StateServiceClient<Channel>> =
    LazyLock::new(|| {
        StateServiceClient::new(
            Endpoint::from_static("http://[::1]:50050").connect_lazy(),
        )
    });

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DhcpSpec {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DhcpDerivedSpec {
    pub link: String,
}

type DhcpWorkState = (CancellationToken, Option<DhcpState>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpState {
    pub address: Ipv4Addr,
    pub prefix_len: u8,
    pub router: Ipv4Addr,
}

impl DhcpReconciler {
    #[must_use]
    pub const fn new_with(handle: Handle) -> Self {
        Self { rtnl: handle }
    }
}

impl DhcpReconciler {
    const MAX_ATTEMPTS: u8 = 5;
    const MAX_NDOTS: u8 = 15;
    const MAX_NS: usize = 3;
    const MAX_SORTLIST: usize = 10;
    const MAX_TIMEOUT: u8 = 30;

    pub async fn validate(
        &self,
        key: Key,
        spec: DhcpSpec,
        resource: Option<DhcpResource>,
    ) -> Result<ValidateResponse<DhcpDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        let deps = Self::get_deps(key.clone())?;
        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(ValidateResponse {
            derived_spec: DhcpDerivedSpec { link: name },
            children: vec![],
            dependencies: deps,
        })
    }

    fn get_deps(key: Key) -> Result<HashSet<Identity>> {
        let name = key.name.ok_or_else(|| anyhow!("missing name"))?;

        Ok(HashSet::from([Identity::Private(
            PrivateIdentity::Dynamic(Key {
                schema: "network:link".to_owned(),
                name: Some(name),
            }),
        )]))
    }

    async fn wait(
        fd: &OwnedFd,
        timestamp: smoltcp::time::Instant,
        iface: &mut Interface,
        sockets: &SocketSet<'_>,
    ) -> std::io::Result<()> {
        let delay = iface.poll_delay(timestamp, sockets);
        let stream = AsyncFd::new(fd.try_clone()?)?;

        if let Some(d) = delay {
            let dur = Duration::from(d);

            match timeout(dur, stream.readable()).await {
                Ok(Ok(mut guard)) => {
                    guard.clear_ready();
                    Ok(())
                }
                Ok(Err(e)) => Err(e),
                Err(_) => Ok(()),
            }
        } else {
            let mut guard = stream.readable().await?;
            guard.clear_ready();
            Ok(())
        }
    }

    fn create_dhcp_client(
        addr: [u8; 6],
        key: Key,
        dev: String,
    ) -> Result<CancellationToken> {
        let mut device = RawSocket::new(&dev, Medium::Ethernet)?;
        let mut config = Config::new(EthernetAddress(addr).into());
        config.random_seed = rand::random();
        let token = CancellationToken::new();
        let ret_token = token.clone();

        tokio::task::spawn_local(async move {
            let mut iface =
                Interface::new(config, &mut device, Instant::now().into());

            let mut socket = dhcpv4::Socket::new();
            socket.set_max_lease_duration(Some(Duration::from_secs(10).into()));

            let mut sockets = SocketSet::new(vec![]);
            let handle = sockets.add(socket);

            // SAFETY: TODO
            let fd = unsafe { OwnedFd::from_raw_fd(device.as_raw_fd()) };

            let ret = loop {
                if token.is_cancelled() {
                    break Ok(());
                }

                let timestamp = Instant::now().into();

                iface.poll_maintenance(timestamp);

                while iface.poll_ingress_single(
                    timestamp,
                    &mut device,
                    &mut sockets,
                ) == PollIngressSingleResult::PacketProcessed
                {
                    yield_now().await;
                }

                iface.poll_egress(timestamp, &mut device, &mut sockets);

                let socket = sockets.get_mut::<dhcpv4::Socket>(handle);
                if let Some(event) = socket.poll() {
                    let mut clients = (*CLIENTS).lock().await;
                    let Some((_, client)) = clients.get_mut(&dev) else {
                        drop(clients);
                        break Err(anyhow!("no clients in client loop"));
                    };

                    match event {
                        dhcpv4::Event::Deconfigured => {
                            client.take();
                        }
                        dhcpv4::Event::Configured(config) => {
                            *client = Some(DhcpState {
                                address: config.address.address(),
                                prefix_len: config.address.prefix_len(),
                                router: config
                                    .router
                                    .unwrap_or(config.server.address),
                            });

                            let mut c = (*STATE_CLIENT).clone();

                            let raw = match serde_json::to_vec(&key) {
                                Ok(v) => v,
                                Err(err) => break Err(anyhow!(err)),
                            };

                            let _ = timeout(
                                Duration::from_secs(1),
                                c.reconcile_now(ReconcileNowRequest { raw }),
                            )
                            .await;
                        }
                    }
                }

                if let Err(err) =
                    Self::wait(&fd, timestamp, &mut iface, &sockets).await
                {
                    break Err(anyhow!(err));
                }
            };

            let mut clients = (*CLIENTS).lock().await;
            clients.remove(&dev);
            drop(clients);

            let _ = dbg!(ret);
        });

        Ok(ret_token)
    }

    pub async fn reconcile(
        &self,
        resource: DhcpResource,
    ) -> Result<ResourceResponse<DhcpState>> {
        let linkinfo = LinkReconciler::get_link_info(
            &self.rtnl,
            resource.derived_spec.link.clone(),
        )
        .await;

        let linkinfo = match linkinfo {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: None,
                    children: vec![],
                    dependencies: Self::get_deps(resource.id.key().clone())?,
                });
            }
        };

        let LinkContext::Link(link) = linkinfo else {
            return Ok(ResourceResponse {
                status: Status::Error("link does not exist".to_owned().into()),
                state: None,
                children: vec![],
                dependencies: Self::get_deps(resource.id.key().clone())?,
            });
        };

        let mut clients = (*CLIENTS).lock().await;
        if matches!(resource.phase, Phase::Shutdown | Phase::Deleting) {
            if let Some(client) = clients.remove(&resource.derived_spec.link) {
                client.0.cancel();
            }

            return Ok(ResourceResponse {
                status: Status::Deleted,
                state: None,
                children: vec![],
                dependencies: Self::get_deps(resource.id.key().clone())?,
            });
        }

        let ret = clients
            .entry(resource.derived_spec.link.clone())
            .or_try_insert_with(|| {
                Self::create_dhcp_client(
                    link.address,
                    resource.id.key().clone(),
                    resource.derived_spec.link.clone(),
                )
                .map(|v| (v, None))
            });

        let cfg = match ret {
            Ok(v) => v.1.clone(),
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}").into()),
                    state: None,
                    children: vec![],
                    dependencies: Self::get_deps(resource.id.key().clone())?,
                });
            }
        };
        drop(clients);

        let addr = cfg
            .clone()
            .map(|v| {
                anyhow::Ok(SubResourceCreate::<Value> {
                    id: Identity::Private(PrivateIdentity::Dynamic(Key {
                        schema: "network:address".to_owned(),
                        name: Some(format!(
                            "{}-dhcp",
                            resource.derived_spec.link
                        )),
                    })),
                    spec: serde_json::to_value(AddressSpec {
                        dev: resource.derived_spec.link.clone(),
                        address: v.address.into(),
                        prefix_len: v.prefix_len,
                    })?,
                })
            })
            .transpose()?;

        let rtr = cfg
            .clone()
            .map(|v| {
                anyhow::Ok(SubResourceCreate::<Value> {
                    id: Identity::Private(PrivateIdentity::Dynamic(Key {
                        schema: "network:route".to_owned(),
                        name: Some(format!(
                            "{}-dhcp",
                            resource.derived_spec.link
                        )),
                    })),
                    spec: serde_json::to_value(RouteSpec::Ipv4 {
                        destination: "0.0.0.0".parse()?,
                        prefix_len: 0,
                        gateway: v.router,
                        parent: Some(format!(
                            "{}-dhcp",
                            resource.derived_spec.link
                        )),
                    })?,
                })
            })
            .transpose()?;

        Ok(ResourceResponse {
            status: Status::Ready,
            state: cfg,
            children: vec![addr, rtr].into_iter().flatten().collect(),
            dependencies: Self::get_deps(resource.id.key().clone())?,
        })
    }

    async fn validate_new_spec(&self, _spec: &DhcpSpec) -> Result<()> {
        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &DhcpResource,
        spec: &DhcpSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }
}
