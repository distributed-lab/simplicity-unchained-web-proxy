use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use config::{Config as ConfigLoader, File};

use api::api::Config;

#[derive(Parser, Debug)]
#[command(version, about = "Simplicity Proxy Service")]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "./config.toml")]
    pub config: PathBuf,
    /// Override the port to bind to
    #[arg(long)]
    pub port: Option<u16>,
    /// Override the host to bind to
    #[arg(long)]
    pub host: Option<String>,
}

impl Cli {
    pub fn build_config(&self) -> Result<Config> {
        let loader = ConfigLoader::builder().add_source(File::from(self.config.clone()));
        let mut config: Config = loader.build()?.try_deserialize()?;

        if let Some(cli_port) = self.port {
            config.port = cli_port;
        }
        if let Some(cli_host) = &self.host {
            config.host = cli_host.clone();
        }

        Ok(config)
    }
}
