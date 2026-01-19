use anyhow::{Context, Result};
use axum::{Json, http::StatusCode};
use elements::{
    EcdsaSighashType,
    bitcoin::PublicKey,
    encode::{deserialize, serialize},
    hashes::Hash,
    pset::PartiallySignedTransaction,
    script::Script,
    secp256k1_zkp::{Message, Secp256k1, SecretKey},
    sighash::SighashCache,
};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

#[derive(Deserialize)]
pub struct SignPsetRequest {
    pub pset_hex: String,
    pub secret_key_hex: String,
    pub input_index: usize,
    pub redeem_script_hex: String,
}

pub async fn sign_pset_handler(
    Json(payload): Json<SignPsetRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let result = execute(
        &payload.pset_hex,
        &payload.secret_key_hex,
        payload.input_index,
        &payload.redeem_script_hex,
    );

    match result {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub fn execute(
    pset_hex: &str,
    secret_key_hex: &str,
    input_index: usize,
    redeem_script_hex: &str,
) -> Result<serde_json::Value> {
    let pset_bytes = hex::decode(pset_hex).context("Failed to decode PSET hex")?;
    let mut pset: PartiallySignedTransaction =
        deserialize(&pset_bytes).context("Failed to deserialize PSET")?;

    if input_index >= pset.inputs().len() {
        return Err(anyhow::anyhow!(
            "Input index {} out of bounds (PSET has {} inputs)",
            input_index,
            pset.inputs().len()
        ));
    }

    let secret_key_bytes =
        hex::decode(secret_key_hex).context("Failed to decode secret key hex")?;
    let secret_key = SecretKey::from_slice(&secret_key_bytes).context("Invalid secret key")?;

    let redeem_script_bytes =
        hex::decode(redeem_script_hex).context("Failed to decode redeem script hex")?;
    let redeem_script = Script::from(redeem_script_bytes);

    let secp = Secp256k1::new();

    let public_key = PublicKey::from_private_key(
        &secp,
        &elements::bitcoin::PrivateKey {
            compressed: true,
            network: elements::bitcoin::NetworkKind::Main,
            inner: secret_key,
        },
    );

    let tx = pset.extract_tx()?;

    let pset_input = &pset.inputs()[input_index];

    let prev_value = pset_input
        .witness_utxo
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing witness UTXO for input {}", input_index))?
        .value;

    // Compute sighash for P2WSH (SegWit v0)
    let mut sighash_cache = SighashCache::new(&tx);
    let sighash = sighash_cache.segwitv0_sighash(
        input_index,
        &redeem_script,
        prev_value,
        EcdsaSighashType::All,
    );

    let msg = Message::from_digest(sighash.to_byte_array());
    let signature = secp.sign_ecdsa(&msg, &secret_key);

    let mut sig_bytes = signature.serialize_der().to_vec();
    sig_bytes.push(EcdsaSighashType::All.as_u32() as u8);

    // Add signature to PSET
    let input = &mut pset.inputs_mut()[input_index];
    input.partial_sigs.insert(public_key, sig_bytes.clone());

    if input.witness_script.is_none() {
        input.witness_script = Some(redeem_script.clone());
    }

    let partial_sigs_count = pset.inputs()[input_index].partial_sigs.len();

    let output = json!({
        "pset": hex::encode(serialize(&pset)),
        "signature_hex": hex::encode(&sig_bytes),
        "public_key_hex": hex::encode(public_key.to_bytes()),
        "input_index": input_index,
        "partial_sigs_count": partial_sigs_count,
    });

    Ok(output)
}
