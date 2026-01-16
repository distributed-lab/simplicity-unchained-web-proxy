use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;
use api::api::define_routes;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = cli.build_config()?;
    define_routes(config).await?;
    Ok(())
}
