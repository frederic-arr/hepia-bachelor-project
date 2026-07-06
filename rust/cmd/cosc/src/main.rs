use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use cos_api_api::ApiResource;
use cos_api_api::proto::v1::PushConfigRequest;
use cos_api_api_client::proto::v1::ApiServiceClient;
use serde::{Deserialize, Serialize};
use tonic::Request;
use tonic::transport::Endpoint;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Push {
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigResource {
    pub schema: String,
    pub name: String,

    #[serde(flatten)]
    pub spec: rmpv::Value,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let conn = Endpoint::from_str(&cli.server).unwrap().connect_lazy();
    let mut client = ApiServiceClient::new(conn);

    match cli.command {
        Commands::Push { config } => {
            let configs = serde_yaml::Deserializer::from_reader(
                std::fs::File::open(config).unwrap(),
            )
            .map(|de| ConfigResource::deserialize(de).unwrap())
            .map(|r| ApiResource {
                schema: format!("config#{}", r.schema),
                name: r.name.clone(),
                spec: rmp_serde::to_vec(&r.spec).unwrap(),
            })
            .collect::<Vec<_>>();

            let request = Request::new(PushConfigRequest {
                raw: rmp_serde::to_vec(&configs).unwrap(),
            });

            client.push_config(request).await.unwrap();
        }
    }
}
