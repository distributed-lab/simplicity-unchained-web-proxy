use anyhow::Result;
use axum::{Json, http::StatusCode};
use elements::bitcoin::PublicKey;
use elements::secp256k1_zkp::{Secp256k1, SecretKey, rand::rngs::OsRng};
use serde_json::{Value, json};

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    let public_key = PublicKey {
        compressed: true,
        inner: public_key,
    };

    (secret_key, public_key)
}

pub async fn generate_keypair_handler() -> Result<Json<Value>, (StatusCode, String)> {
    match execute() {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub fn execute() -> Result<Value> {
    let (secret_key, public_key) = generate_keypair();

    let output = json!({
        "secret_key": hex::encode(secret_key.secret_bytes()),
        "public_key": hex::encode(public_key.to_bytes()),
        "compressed": public_key.compressed
    });

    Ok(output)
}
