use std::sync::Arc;

use clap::Parser;
use futures::stream::StreamExt;
use openc2::{Action, Feature, json::Command};
use openc2_consumer::{Consume, Registry};

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
    }

    Ok(())
}
