use anyhow::{Context, Result};
use axum::{Json, http::StatusCode};
use elements::{
    EcdsaSighashType, encode::deserialize, hashes::Hash, pset::PartiallySignedTransaction,
    script::Script, sighash::SighashCache,
};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

#[derive(Deserialize)]
pub struct SighashPsetRequest {
    pub pset_hex: String,
    pub input_index: usize,
    pub redeem_script_hex: String,
}

pub async fn sighash_pset_handler(
    Json(payload): Json<SighashPsetRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let result = execute(
        &payload.pset_hex,
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
    input_index: usize,
    redeem_script_hex: &str,
) -> Result<serde_json::Value> {
    let pset_bytes = hex::decode(pset_hex).context("Failed to decode PSET hex")?;
    let pset: PartiallySignedTransaction =
        deserialize(&pset_bytes).context("Failed to deserialize PSET")?;

    if input_index >= pset.inputs().len() {
        return Err(anyhow::anyhow!(
            "Input index {} out of bounds (PSET has {} inputs)",
            input_index,
            pset.inputs().len()
        ));
    }

    let redeem_script_bytes =
        hex::decode(redeem_script_hex).context("Failed to decode redeem script hex")?;
    let redeem_script = Script::from(redeem_script_bytes);

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

    let output = json!({
        "sighash_hex": hex::encode(sighash.as_byte_array()),
        "message_hex": hex::encode(sighash.as_byte_array()),
        "input_index": input_index,
        "sighash_type": "SIGHASH_ALL",
    });

    Ok(output)
}
