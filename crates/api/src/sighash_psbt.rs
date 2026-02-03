use anyhow::{Context, Result};
use axum::{Json, http::StatusCode};
use elements::bitcoin::{EcdsaSighashType, hashes::Hash, psbt::Psbt, sighash::SighashCache};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct SighashPsbtRequest {
    pub psbt_hex: String,
    pub input_index: usize,
    pub redeem_script_hex: String,
}

pub async fn sighash_psbt_handler(
    Json(payload): Json<SighashPsbtRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let result = execute(
        &payload.psbt_hex,
        payload.input_index,
        &payload.redeem_script_hex,
    );

    match result {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub fn execute(
    psbt_hex: &str,
    input_index: usize,
    redeem_script_hex: &str,
) -> Result<serde_json::Value> {
    let psbt_bytes = hex::decode(psbt_hex).context("Failed to decode PSBT hex")?;
    let psbt: Psbt = Psbt::deserialize(&psbt_bytes).context("Failed to deserialize PSBT")?;

    if input_index >= psbt.inputs.len() {
        return Err(anyhow::anyhow!(
            "Input index {} out of bounds (PSBT has {} inputs)",
            input_index,
            psbt.inputs.len()
        ));
    }

    let redeem_script_bytes =
        hex::decode(redeem_script_hex).context("Failed to decode redeem script hex")?;
    let redeem_script = elements::bitcoin::ScriptBuf::from_bytes(redeem_script_bytes);

    let psbt_input = &psbt.inputs[input_index];
    let prev_value = psbt_input
        .witness_utxo
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing witness UTXO for input {}", input_index))?
        .value;

    // Clone psbt to extract transaction, since extract_tx consumes it
    let tx = psbt.clone().extract_tx_unchecked_fee_rate();

    // Compute sighash for P2WSH (SegWit v0)
    let mut sighash_cache = SighashCache::new(&tx);
    let sighash = sighash_cache
        .p2wsh_signature_hash(
            input_index,
            &redeem_script,
            prev_value,
            EcdsaSighashType::All,
        )
        .context("Failed to compute sighash")?;

    let output = json!({
        "sighash_hex": hex::encode(sighash.as_byte_array()),
        "message_hex": hex::encode(sighash.as_byte_array()),
        "input_index": input_index,
        "sighash_type": "SIGHASH_ALL",
    });

    Ok(output)
}
