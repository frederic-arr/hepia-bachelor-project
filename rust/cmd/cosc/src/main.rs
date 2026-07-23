#![expect(clippy::print_stdout, reason = "TODO")]

use std::path::PathBuf;
use std::str::FromStr as _;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cos_proto_api::v1::{
    GetResourceRequest,
    ListResourcesRequest,
    PushConfigRequest,
    ReconcileNowRequest,
};
use cos_proto_api_client::v1::ApiServiceClient;
use cos_proto_reconciler::{
    Identity,
    Key,
    PrivateIdentity,
    SubResourceCreate,
    TerminalResource,
};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tonic::Request;
use tonic::transport::Endpoint;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    server: String,

    #[arg(short, long)]
    password: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Reconcile {
        schema: String,
        name: Option<String>,
    },
    Push {
        #[arg(short, long)]
        config: PathBuf,
    },
    List {},
    Get {
        schema: String,
        name: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigResource {
    pub schema: String,
    pub name: Option<String>,

    #[serde(flatten)]
    pub spec: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayResource {
    pub schema: String,
    pub name: Option<String>,

    pub spec: Value,
    pub derived: Value,
    pub state: Option<Value>,
    pub dependencies: Vec<String>,
    pub children: Vec<String>,
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let conn = Endpoint::from_str(&cli.server)?.connect_lazy();
    let mut client = ApiServiceClient::new(conn);

    match cli.command {
        Commands::Reconcile { name, schema } => {
            let raw = serde_json::to_vec(&Key { name, schema })?;

            let mut request = Request::new(ReconcileNowRequest { raw });
            if let Some(password) = cli.password {
                request.metadata_mut().append("x-auth", password.parse()?);
            }

            let _ = client.reconcile_now(request).await?;
        }
        Commands::Push { config } => {
            let configs = serde_yaml::Deserializer::from_reader(
                std::fs::File::open(config)?,
            )
            .map(ConfigResource::deserialize)
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(|v| SubResourceCreate {
                id: Identity::Private(PrivateIdentity::Static(Key {
                    schema: v.schema,
                    name: v.name,
                })),
                spec: v.spec,
            })
            .collect::<Vec<_>>();

            let mut request = Request::new(PushConfigRequest {
                raw: serde_json::to_vec(&configs)?,
            });
            if let Some(password) = cli.password {
                request.metadata_mut().append("x-auth", password.parse()?);
            }

            client.push_config(request).await?;
        }
        Commands::List {} => {
            let mut request = Request::new(ListResourcesRequest { raw: vec![] });
            if let Some(password) = cli.password {
                request.metadata_mut().append("x-auth", password.parse()?);
            }

            let raw = client.list_resources(request).await?.into_inner().raw;
            let resources = serde_json::from_slice::<
                Vec<TerminalResource<Value, Value, Value>>,
            >(&raw)?
            .into_iter()
            .map(|v| DisplayResource {
                schema: v.id.schema().clone(),
                name: v.id.key().name.clone(),
                spec: v.spec,
                derived: v.derived_spec,
                state: v.state,
                dependencies: v
                    .dependencies
                    .into_iter()
                    .map(|v| format!("{v}"))
                    .collect_vec(),
                children: v
                    .children
                    .into_iter()
                    .map(|v| format!("{v}"))
                    .collect_vec(),
            })
            .collect_vec();

            let v = serde_json::to_string(&resources)?;
            println!("{v}");
        }
        Commands::Get { name, schema } => {
            let raw = serde_json::to_vec(&Key { name, schema })?;

            let mut request = Request::new(GetResourceRequest { raw });
            if let Some(password) = cli.password {
                request.metadata_mut().append("x-auth", password.parse()?);
            }

            let raw = client.get_resource(request).await?.into_inner().raw;
            let resource = serde_json::from_slice::<
                TerminalResource<Value, Value, Value>,
            >(&raw)
            .map(|v| DisplayResource {
                schema: v.id.schema().clone(),
                name: v.id.key().name.clone(),
                spec: v.spec,
                derived: v.derived_spec,
                state: v.state,
                dependencies: v
                    .dependencies
                    .into_iter()
                    .map(|v| format!("{v}"))
                    .collect_vec(),
                children: v
                    .children
                    .into_iter()
                    .map(|v| format!("{v}"))
                    .collect_vec(),
            })?;

            let v = serde_json::to_string(&resource)?;
            println!("{v}");
        }
    }

    Ok(())
}
