#![feature(hash_map_macro)]

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::{debug_assert_matches, hash_map};

use anyhow::{Result, anyhow};
use cos_proto_reconciler::v1::{ReconcileRequest, ValidateRequest};
use cos_proto_reconciler::{
    Identity,
    Key,
    Phase,
    Resource,
    ResourceResponse,
    Status,
    TerminalResource,
    ValidateResponse,
};
use cos_proto_reconciler_client::v1::ReconcilerServiceClient;
use serde_json::{Value, json};
use tonic::transport::{Channel, Endpoint};

fn default_config() -> Vec<Resource<Value, Option<Value>, Option<Value>>> {
    vec![Resource::<Value, Option<Value>, Option<Value>> {
        id: Identity::Static(Key {
            schema: "network:dns".to_owned(),
            name: None,
        }),
        phase: Phase::Running,
        status: Status::Unknown,
        spec: json!({
            "nameservers": ["9.9.9.9"],
        }),
        derived_spec: None,
        state: None,
        children: vec![],
        dependencies: vec![],
        dependents: vec![],
    }]
}

async fn get_clients()
-> Result<HashMap<String, ReconcilerServiceClient<Channel>>> {
    let system_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50051").connect_lazy(),
    );

    let network_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50052").connect_lazy(),
    );

    let container_client = ReconcilerServiceClient::new(
        Endpoint::from_static("http://[::1]:50053").connect_lazy(),
    );

    Ok(hash_map! {
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
    })
}

type StoredResource = TerminalResource<Value, Value, Value>;

#[tokio::main]
#[expect(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let clients = get_clients().await?;
    let mut resources: HashMap<Key, StoredResource> = HashMap::new();

    tracing::info!("creating default config");
    let start = Instant::now();
    for resource in default_config() {
        tracing::info!(resource = %resource.id.key(), "creating resource");
        let start = Instant::now();
        let mut client =
            clients.get(resource.id.schema()).cloned().ok_or_else(|| {
                anyhow!("no clients for {}", resource.id.schema())
            })?;

        let request = tonic::Request::new(ValidateRequest {
            raw: serde_json::to_vec(&resource)?,
        });

        let response = client.validate(request).await?.into_inner();
        let response: ValidateResponse<Value> =
            serde_json::from_slice(&response.raw)?;

        let previous = resources.insert(
            resource.id.key().clone(),
            StoredResource {
                id: resource.id,
                phase: resource.phase,
                status: resource.status,
                spec: resource.spec,
                derived_spec: response.derived_spec,
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            },
        );
        tracing::info!(elapsed = ?start.elapsed(), "resource created");

        debug_assert_matches!(previous, None);
    }
    tracing::info!(elapsed = ?start.elapsed(), "default config created");

    loop {
        let mut batch = vec![];

        let c = resources.clone();
        for resource in resources.values_mut() {
            if resource.status == Status::Done {
                continue;
            }

            tracing::info!(resource = %resource.id.key(), "reconciling resource");
            let start = Instant::now();
            let mut client =
                clients.get(resource.id.schema()).cloned().ok_or_else(
                    || anyhow!("no clients for {}", resource.id.schema()),
                )?;

            let request = tonic::Request::new(ReconcileRequest {
                raw: serde_json::to_vec(&Resource::<Value, Value, Value> {
                    id: resource.id.clone(),
                    phase: resource.phase.clone(),
                    status: resource.status.clone(),
                    spec: resource.spec.clone(),
                    derived_spec: resource.derived_spec.clone(),
                    state: resource.state.clone(),
                    children: resource
                        .children
                        .iter()
                        .filter_map(|id| c.get(id.key()))
                        .cloned()
                        .collect(),
                    dependencies: resource
                        .dependencies
                        .iter()
                        .filter_map(|id| c.get(id.key()))
                        .cloned()
                        .collect(),
                    dependents: resource
                        .dependents
                        .iter()
                        .filter_map(|id| c.get(id.key()))
                        .cloned()
                        .collect(),
                })?,
            });

            let response = client.reconcile(request).await?.into_inner();
            let mut response: ResourceResponse<Value> =
                serde_json::from_slice(&response.raw)?;

            resource.status = response.status;
            resource.state = response.state;
            resource.dependencies = response.dependencies;
            resource.children =
                response.children.iter().map(|c| c.id.clone()).collect();

            batch.append(&mut response.children);
            tracing::info!(elapsed = ?start.elapsed(), "resource reconciled");
        }

        for resource in batch {
            let mut resource = match resources.get(resource.id.key()).cloned() {
                Some(mut r) => {
                    if r.spec == resource.spec {
                        continue;
                    }

                    r.status = Status::Unknown;
                    r.spec = resource.spec;
                    r
                }
                None => StoredResource {
                    id: resource.id,
                    phase: Phase::Running,
                    status: Status::Unknown,
                    spec: resource.spec,
                    derived_spec: Value::Null,
                    state: None,
                    children: vec![],
                    dependencies: vec![],
                    dependents: vec![],
                },
            };

            tracing::info!(resource = %resource.id.key(), "updating resource");
            let start = Instant::now();

            let mut client =
                clients.get(resource.id.schema()).cloned().ok_or_else(
                    || anyhow!("no clients for {}", resource.id.schema()),
                )?;

            let request = tonic::Request::new(ValidateRequest {
                raw: serde_json::to_vec(&resource)?,
            });

            let response = client.validate(request).await?.into_inner();
            let response: ValidateResponse<Value> =
                serde_json::from_slice(&response.raw)?;

            resource.derived_spec = response.derived_spec;

            resources.insert(resource.id.key().clone(), resource);
            tracing::info!(elapsed = ?start.elapsed(), "resource upated");
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
