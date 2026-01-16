use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{Router, routing::post};
use serde::Deserialize;

use crate::{compiler::compile_handler, converter::convert_handler};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub async fn define_routes(config: Config) -> Result<()> {
    let router = Router::new()
        .route("/compile", post(compile_handler))
        .route("/convert", post(convert_handler));

    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str
        .parse()
        .context("Invalid address string in config")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
