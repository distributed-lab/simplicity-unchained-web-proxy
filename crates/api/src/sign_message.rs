use anyhow::{Context, Result};
use axum::{Json, http::StatusCode};
use elements::{
    bitcoin::PublicKey,
    secp256k1_zkp::{Message, Secp256k1, SecretKey},
};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
pub struct SignHexRequest {
    pub message: String,
    pub secret_key_hex: String,
}

pub async fn sign_hex_handler(
    Json(payload): Json<SignHexRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let result = execute_sign_hex(&payload.message, &payload.secret_key_hex);

    match result {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub fn execute_sign_hex(message: &str, secret_key_hex: &str) -> Result<serde_json::Value> {
    let secret_key_bytes =
        hex::decode(secret_key_hex).context("Failed to decode secret key hex")?;
    let secret_key = SecretKey::from_slice(&secret_key_bytes).context("Invalid secret key")?;

    let message = hex::decode(message)?;

    let mut hasher = Sha256::new();
    hasher.update(message);
    let digest_hex = hex::encode(hasher.finalize());

    let digest_bytes = hex::decode(&digest_hex).context("Failed to decode digest hex")?;
    if digest_bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Digest must be exactly 32 bytes for secp256k1 Message, got {} bytes",
            digest_bytes.len()
        ));
    }
    let mut digest_arr = [0u8; 32];
    digest_arr.copy_from_slice(&digest_bytes);

    let secp = Secp256k1::new();
    let msg = Message::from_digest(digest_arr);
    let signature = secp.sign_ecdsa(&msg, &secret_key);

    let sig_bytes = signature.serialize_der().to_vec();

    let public_key = PublicKey::from_private_key(
        &secp,
        &elements::bitcoin::PrivateKey {
            compressed: true,
            network: elements::bitcoin::NetworkKind::Main,
            inner: secret_key,
        },
    );

    let output = json!({
        "signature_hex": hex::encode(&sig_bytes),
        "public_key_hex": hex::encode(public_key.to_bytes()),
        "digest_hex": digest_hex,
    });

    Ok(output)
}
