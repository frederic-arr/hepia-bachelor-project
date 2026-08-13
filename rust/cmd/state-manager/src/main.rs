#![feature(exit_status_error)]
#![feature(hash_map_macro)]

mod api;
mod queue;
mod state;
mod timeout;

use std::collections::{HashMap, HashSet};
use std::hash_map;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context as _, Result};
use cos_proto_api_server::v1::ApiServiceServer;
use cos_proto_reconciler::{
    Identity,
    Key,
    PrivateIdentity,
    SubResourceCreate,
    TerminalResource,
    ValidateResponse,
};
use cos_proto_reconciler_client::v1::ReconcilerServiceClient;
use cos_proto_state::v1::{ReconcileNowRequest, ReconcileNowResponse};
use cos_proto_state_server::v1::{StateService, StateServiceServer};
use itertools::Itertools as _;
use network_controller::{
    DhcpSpec,
    DnsSpec,
    LinkSpec,
    LinkSpecType,
    LinkSpecUnspec,
};
use serde_json::Value;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Channel, Endpoint, Server};
use tonic::{Request, Response, Status};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

use crate::api::{ApiAuth, ApiConfig, ApiServer};
use crate::queue::Queue;
use crate::state::StateManager;

#[expect(clippy::unwrap_used, reason = "this is early in the program")]
fn default_config() -> Vec<SubResourceCreate<Value>> {
    vec![
        SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Static(Key {
                schema: "api".to_owned(),
                name: None,
            })),
            spec: serde_json::to_value(ApiConfig {
                auth: ApiAuth::None,
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Static(Key {
                schema: "network:dns".to_owned(),
                name: None,
            })),
            spec: serde_json::to_value(DnsSpec {
                nameservers: vec!["9.9.9.9".to_owned()],
                ..Default::default()
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Static(Key {
                schema: "network:link".to_owned(),
                name: Some("eth0".to_owned()),
            })),
            spec: serde_json::to_value(LinkSpec {
                admin_up: true,
                link_type: LinkSpecType::Unspec(LinkSpecUnspec {}),
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Private(PrivateIdentity::Static(Key {
                schema: "network:dhcp".to_owned(),
                name: Some("eth0".to_owned()),
            })),
            spec: serde_json::to_value(DhcpSpec {}).unwrap(),
        },
    ]
}

fn get_clients() -> HashMap<String, ReconcilerServiceClient<Channel>> {
    let system_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50051").connect_lazy(),
    );

    let network_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50052").connect_lazy(),
    );

    let container_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50053").connect_lazy(),
    );

    hash_map! {
        // System resources
        "system:static-file".to_owned() => system_client,

        // Network resources
        "network:dns".to_owned() => network_client.clone(),
        "network:interface".to_owned() => network_client.clone(),
        "network:link".to_owned() => network_client.clone(),
        "network:route".to_owned() => network_client.clone(),
        "network:address".to_owned() => network_client.clone(),
        "network:dhcp".to_owned() => network_client,

        // Container resources
        "container:runtime".to_owned() => container_client.clone(),
        "container:instance".to_owned() => container_client.clone(),
        "container:image".to_owned() => container_client.clone(),
        "container:network".to_owned() => container_client,
    }
}

struct StateManagerService {
    sm: Arc<StateManager>,
}

#[tonic::async_trait]
impl StateService for StateManagerService {
    async fn reconcile_now(
        &self,
        request: Request<ReconcileNowRequest>,
    ) -> Result<Response<ReconcileNowResponse>, Status> {
        let req = request.into_inner();
        let key: Key = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        tracing::info!(%key, "reconciliation triggered from external source");

        self.sm.queue.schedule_at(key, Instant::now()).await;

        Ok(Response::new(ReconcileNowResponse {
            raw: vec![],
        }))
    }
}

async fn create_default_state_manager() -> Result<(
    StateManager,
    TerminalResource<Value, Value, Value>,
)> {
    tracing::info!("loading default config");
    let mut resources = HashMap::new();
    let clients = get_clients();
    let queue = Queue::new();

    tracing::info!("creating default config");
    let start = Instant::now();
    StateManager::bulk_upsert(
        &clients,
        &queue,
        &mut resources,
        HashSet::new(),
        default_config()
            .into_iter()
            .map(|v| (v.id.key().clone(), v))
            .collect(),
        true,
    )
    .await?;
    tracing::info!(elapsed = ?start.elapsed(), "default config created");

    let config = resources
        .get(&Key {
            schema: "api".to_owned(),
            name: None,
        })
        .context("an api configuration must exist")?
        .clone();

    let sm = StateManager::new(clients, resources, queue).await;
    Ok((sm, config))
}

async fn create_state_manager_from_disk() -> Result<(
    StateManager,
    TerminalResource<Value, Value, Value>,
)> {
    tracing::info!("loading from disk");
    let clients = get_clients();
    let queue = Queue::new();

    let bundle = std::fs::read_to_string("/config/bundle.json")?;
    let resources = StateManager::from_bundle(bundle.as_bytes())?;
    tracing::info!("loaded {} resources from disk", resources.len());
    let config = resources
        .get(&Key {
            schema: "api".to_owned(),
            name: None,
        })
        .context("an api configuration must exist")?
        .clone();

    let keys = resources
        .values()
        .map(|v| {
            (
                SubResourceCreate::<Value> {
                    id: v.id.clone(),
                    spec: v.spec.clone(),
                },
                ValidateResponse::<Value> {
                    derived_spec: v.derived_spec.clone(),
                    children: vec![],
                    dependencies: v.dependencies.clone(),
                },
            )
        })
        .collect_vec();

    StateManager::schedule_available(keys.iter(), &queue, &resources, true)
        .await?;

    let sm = StateManager::new(clients, resources, queue).await;
    Ok((sm, config))
}

struct ImmediateWriter;

impl<'writer> MakeWriter<'writer> for ImmediateWriter {
    type Writer = ImmediateStdout;

    fn make_writer(&'writer self) -> Self::Writer {
        ImmediateStdout(std::io::stdout())
    }
}

struct ImmediateStdout(std::io::Stdout);

impl Write for ImmediateStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.0.write(buf)?;
        self.0.flush()?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(buf)?;
        self.0.flush()
    }
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(ImmediateWriter)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .parse_lossy("state_manager=info"),
        )
        .init();

    let (sm, config) = match create_state_manager_from_disk().await {
        Ok(v) => v,
        Err(_) => create_default_state_manager().await?,
    };
    let ct = CancellationToken::new();
    let reconciliation_ct = ct.clone();

    #[expect(clippy::unwrap_used, reason = "this is early in the program")]
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = ctrl_c() => tracing::info!("recieved CTRL+C"),
            _ = sigterm.recv() => tracing::info!("recieved SIGTERM"),
            () = ct.cancelled() => tracing::info!("recieved internal shutdown signal"),
        }

        tracing::info!("shutting down...");
        ct.cancel();
    });

    let sm = Arc::new(sm);
    let addr = "127.0.0.1:50050".parse()?;
    let server = Server::builder()
        .add_service(StateServiceServer::new(StateManagerService {
            sm: Arc::clone(&sm),
        }))
        .serve(addr);

    let addr2 = "0.0.0.0:50000".parse()?;

    let mut cmdline = vec![];
    let data = std::fs::read_to_string("/proc/cmdline")?;
    for v in data.split(' ') {
        match v.split_once('=') {
            Some((k, v)) => cmdline.push((k.to_owned(), v.to_owned())),
            None => cmdline.push((v.to_owned(), String::new())),
        }
    }

    let api = Server::builder()
        .add_service(ApiServiceServer::new(ApiServer {
            sm: Arc::clone(&sm),
            config: Mutex::new(serde_json::from_value(config.spec)?),
            cmdline: cmdline.iter().cloned().collect(),
        }))
        .serve(addr2);

    let rloop = sm.reconciliation_loop(&reconciliation_ct);

    tokio::select! {
        () = rloop => {},
        _ = server => {},
        _ = api => {},
    };

    tracing::info!("saving data to disk");
    // TODO
    tracing::info!("shutdown complete");

    Ok(())
}
