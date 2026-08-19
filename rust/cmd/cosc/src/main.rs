#![expect(clippy::print_stdout, reason = "TODO")]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cos_proto_api::ConfigResource;
use cos_proto_reconciler::{Identity, Key, PrivateIdentity, SubResourceCreate};
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
    command: CommandGroup,
}

#[derive(Debug, Subcommand)]
enum CommandGroup {
    #[command(subcommand)]
    Config(ConfigCommand),
    #[command(subcommand)]
    Fs(FsCommand),
    #[command(subcommand)]
    Resources(ResourcesCommand),
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Pull,
    Validate { config: PathBuf },
    Push { config: PathBuf },
}

#[derive(Debug, Subcommand)]
enum FsCommand {
    Write { content: PathBuf, target: String },
    List { target: String },
    Read { target: String },
    Delete { target: String },
}

#[derive(Debug, Subcommand)]
enum ResourcesCommand {
    List {
        schema: Option<String>,
    },
    Get {
        schema: String,
        name: Option<String>,
    },
    ReconcileNow {
        schema: String,
        name: Option<String>,
    },
    ForceDelete {
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
        CommandGroup::Config(ConfigCommand::Validate { config }) => {
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

            client.config_validate(configs).await?;
        }
        CommandGroup::Config(ConfigCommand::Push { config }) => {
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

            client.config_push(&configs).await?;
        }
        CommandGroup::Config(ConfigCommand::Pull) => {
            let mut ser = serde_yaml::Serializer::new(std::io::stdout());
            let resources = client.config_pull().await?.into_iter().map(|v| {
                ConfigResource {
                    schema: v.id.schema().clone(),
                    name: v.id.key().name.clone(),
                    spec: v.spec.clone(),
                }
            });

            for res in resources {
                res.serialize(&mut ser)?;
            }
        }

        CommandGroup::Fs(FsCommand::Write { .. }) => {
            todo!()
        }

        CommandGroup::Fs(FsCommand::List { .. }) => {
            todo!()
        }

        CommandGroup::Fs(FsCommand::Read { .. }) => {
            todo!()
        }

        CommandGroup::Fs(FsCommand::Delete { .. }) => {
            todo!()
        }

        CommandGroup::Resources(ResourcesCommand::List { schema }) => {
            let resources = client.resources_list().await?;
            let resources = resources
                .into_iter()
                .filter(|v| schema.clone().is_none_or(|s| v.id.schema() == &s))
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

        CommandGroup::Resources(ResourcesCommand::Get { schema, name }) => {
            let resource = client.resources_get(&Key { schema, name }).await;
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

        CommandGroup::Resources(ResourcesCommand::ReconcileNow {
            schema,
            name,
        }) => {
            client
                .resources_reconcile_now(&Key { schema, name })
                .await?;
        }

        CommandGroup::Resources(ResourcesCommand::ForceDelete { .. }) => {
            todo!()
        }
    }

    Ok(())
}
