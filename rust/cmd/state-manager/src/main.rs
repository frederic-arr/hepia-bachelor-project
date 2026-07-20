#![feature(hash_map_macro)]

mod queue;
mod state;
mod timeout;

use std::collections::{HashMap, HashSet};
use std::hash_map;
use std::time::Instant;

use anyhow::Result;
use cos_proto_reconciler::{Identity, Key, SubResourceCreate};
use cos_proto_reconciler_client::v1::ReconcilerServiceClient;
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
use tonic::transport::{Channel, Endpoint};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::queue::Queue;
use crate::state::StateManager;

#[expect(clippy::unwrap_used, reason = "this is early in the program")]
fn default_config() -> Vec<SubResourceCreate<Value>> {
    vec![
        SubResourceCreate::<Value> {
            id: Identity::Static(Key {
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
            id: Identity::Static(Key {
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
            id: Identity::Static(Key {
                schema: "network:dhcp".to_owned(),
                name: Some("eth0".to_owned()),
            }),
            spec: serde_json::to_value(DhcpSpec {
                link: "eth0".to_owned(),
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
        "container:container".to_owned() => container_client.clone(),
        "container:image".to_owned() => container_client,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .parse_lossy("state_manager=trace"),
        )
        .init();

    let mut resources = HashMap::new();
    let clients = get_clients();

    tracing::info!("creating default config");
    let start = Instant::now();
    StateManager::bulk_upsert(
        &clients,
        &Queue::new(),
        &mut resources,
        HashSet::new(),
        default_config()
            .into_iter()
            .map(|v| (v.id.key().clone(), v))
            .collect(),
    )
    .await?;
    tracing::info!(elapsed = ?start.elapsed(), "default config created");

    let sm = StateManager::new(clients, resources).await;
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

    sm.reconciliation_loop(&reconciliation_ct).await;

    tracing::info!("saving data to disk");
    // TODO
    tracing::info!("shutdown complete");

    Ok(())
}
