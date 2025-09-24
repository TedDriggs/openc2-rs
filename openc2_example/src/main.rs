use std::sync::Arc;

use clap::Parser;
use futures::stream::StreamExt;
use openc2::{
    Action, Args, Feature, Nsid,
    json::Command,
    target::{self, Device},
};
use openc2_consumer::{Consume, Registry};
use openc2_er::DownstreamDevice;

mod api;
mod crowdstrike;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Send a `query features` command and show the response.
    QueryFeatures,
    DeleteFile {
        /// The file path to delete.
        #[clap(long)]
        file_path: String,
        /// The target device ID (AID).
        #[clap(long, num_args(1..))]
        aid: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut registry = Registry::default();

    let client = Arc::new(api::crowdstrike::Client::new("", "PLACEHOLDER"));
    // We can use Arc here so the same consumer can be cheaply shared among multiple registries.
    let contain = Arc::new(crowdstrike::EndpointResponse::new(client.clone()));
    // Don't use Arc here just to demonstrate that the registry can take ownership of any consumer.
    let detonate = crowdstrike::Sandbox::new(client.clone());

    registry.add(contain);
    registry.add(detonate);

    match cli.command {
        Subcommand::QueryFeatures => {
            let rsp = registry
                .consume(
                    Command::new(
                        Action::Query,
                        vec![Feature::Pairs, Feature::Profiles, Feature::Versions],
                    )
                    .into(),
                )
                .next()
                .await
                .transpose()?
                .expect("stream yields at least one response");

            println!("{}", serde_json::to_string_pretty(&rsp)?);
        }
        Subcommand::DeleteFile { file_path, aid } => {
            let rsp = registry.consume(
                Command::new(
                    Action::Delete,
                    target::File {
                        path: Some(file_path),
                        ..Default::default()
                    },
                )
                .with_args(Args::try_with_extension(
                    Nsid::ER,
                    &openc2_er::Args {
                        downstream_device: Some(DownstreamDevice {
                            devices: aid.into_iter().map(Device::with_device_id).collect(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )?)
                .into(),
            );

            rsp.inspect(|res| match res {
                Ok(r) => println!("Response: {}", serde_json::to_string_pretty(&r).unwrap()),
                Err(e) => eprintln!("Error: {}", e),
            })
            .collect::<Vec<_>>()
            .await;
        }
    }

    Ok(())
}
