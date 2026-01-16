use ::api::api::define_routes;
use anyhow::Result;
mod cli;
use crate::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = cli.build_config()?;
    define_routes(config).await?;
    Ok(())
}
