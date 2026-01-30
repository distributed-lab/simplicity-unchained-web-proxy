use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{Router, routing::post};
use serde::Deserialize;

use crate::{
    compiler::compile_handler, converter::convert_handler, create::create_pset_handler,
    create_psbt::create_psbt_handler, finalize_psbt::finalize_psbt_handler,
    finilize::finalize_handler, generate::generate_keypair_handler, sign::sign_pset_handler,
    sign_message::sign_hex_handler, sign_psbt::sign_psbt_handler,
};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub async fn define_routes(config: Config) -> Result<()> {
    let router = Router::new()
        .route(
            "/simplicity-unchained-web-proxy-demo/compile",
            post(compile_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/convert",
            post(convert_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/create-pset",
            post(create_pset_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/sign-pset",
            post(sign_pset_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/finalize",
            post(finalize_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/generate",
            post(generate_keypair_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/sign_message",
            post(sign_hex_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/create-psbt",
            post(create_psbt_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/sign-psbt",
            post(sign_psbt_handler),
        )
        .route(
            "/simplicity-unchained-web-proxy-demo/finalize-psbt",
            post(finalize_psbt_handler),
        );

    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str
        .parse()
        .context("Invalid address string in config")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
