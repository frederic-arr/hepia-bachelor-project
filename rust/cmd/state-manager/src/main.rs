#![feature(hash_map_macro)]

mod queue;
mod state;
mod timeout;

use std::collections::{HashMap, HashSet};
use std::hash_map;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use container_controller::RuntimeSpec;
use cos_proto_reconciler::{Identity, Key, SubResourceCreate};
use cos_proto_reconciler_client::v1::ReconcilerServiceClient;
use cos_proto_state::v1::{ReconcileNowRequest, ReconcileNowResponse};
use cos_proto_state_server::v1::{StateService, StateServiceServer};
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
use tokio_util::sync::CancellationToken;
use tonic::transport::{Channel, Endpoint, Server};
use tonic::{Request, Response, Status};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::queue::Queue;
use crate::state::StateManager;

#[expect(clippy::unwrap_used, reason = "this is early in the program")]
fn default_config() -> Vec<SubResourceCreate<Value>> {
    vec![
        SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "network:dns".to_owned(),
                name: None,
            }),
            spec: serde_json::to_value(DnsSpec {
                nameservers: vec!["9.9.9.9".to_owned()],
                ..Default::default()
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "network:link".to_owned(),
                name: Some("eth0".to_owned()),
            }),
            spec: serde_json::to_value(LinkSpec {
                name: "eth0".to_owned(),
                admin_up: true,
                link_type: LinkSpecType::Unspec(LinkSpecUnspec {}),
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "network:dhcp".to_owned(),
                name: Some("eth0".to_owned()),
            }),
            spec: serde_json::to_value(DhcpSpec {
                link: "eth0".to_owned(),
            })
            .unwrap(),
        },
        SubResourceCreate::<Value> {
            id: Identity::Dynamic(Key {
                schema: "container:runtime".to_owned(),
                name: Some("default".to_owned()),
            }),
            spec: serde_json::to_value(RuntimeSpec {
                name: "rootfull".to_owned(),
                engine: "podman".to_owned(),
                uid: 0,
                gid: 0,
                depends_on: HashSet::from([Identity::Dynamic(Key {
                    schema: "network:route".to_owned(),
                    name: Some("eth0-dhcp".to_owned()),
                })]),
            })
            .unwrap(),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .parse_lossy("state_manager=info"),
        )
        .init();

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
    )
    .await?;
    tracing::info!(elapsed = ?start.elapsed(), "default config created");

    let sm = StateManager::new(clients, resources, queue).await;
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
    let addr = "[::1]:50050".parse()?;
    let server = Server::builder()
        .add_service(StateServiceServer::new(StateManagerService {
            sm: Arc::clone(&sm),
        }))
        .serve(addr);

    let rloop = sm.reconciliation_loop(&reconciliation_ct);

    tokio::select! {
        () = rloop => {},
        _ = server => {},
    };

    tracing::info!("saving data to disk");
    // TODO
    tracing::info!("shutdown complete");

    Ok(())
}
