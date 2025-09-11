use std::sync::Arc;

use clap::Parser;
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
    let contain = crowdstrike::EndpointResponse::new(client.clone());
    let detonate = crowdstrike::Sandbox::new(client.clone());

    registry.insert(contain);
    registry.insert(detonate);

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
                .await?;
            println!("{}", serde_json::to_string_pretty(&rsp)?);
        }
    }

    Ok(())
}
