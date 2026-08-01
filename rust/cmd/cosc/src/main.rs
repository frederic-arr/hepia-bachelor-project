#![expect(clippy::print_stdout, reason = "TODO")]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cos_proto_reconciler::{Identity, Key, PrivateIdentity, SubResourceCreate};
use cosc::ConfigResource;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    let mut client = cosc::CosClient::new(&cli.server, cli.password)?;

    match cli.command {
        Commands::Reconcile { name, schema } => {
            client.reconcile(&Key { schema, name }).await?;
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

            client.push(&configs).await?;
        }
        Commands::List {} => {
            let resources = client.list().await?;
            let resources = resources
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
            let resource = client.get_resource(&Key { name, schema }).await;
            let resource = resource.map(|v| DisplayResource {
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
