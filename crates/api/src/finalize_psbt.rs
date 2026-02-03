use anyhow::{Context, Result};
use axum::Json;
use elements::bitcoin::{PublicKey, ScriptBuf, psbt::Psbt};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct FinalizePsbtRequest {
    pub psbt_hex: String,
    pub redeem_script_hex: String,
    pub input_index: usize,
    pub signature_hex: String,
    pub public_key_hex: String,
}

pub async fn finalize_psbt_handler(
    Json(payload): Json<FinalizePsbtRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    match execute(
        &payload.psbt_hex,
        &payload.redeem_script_hex,
        payload.input_index,
        &payload.signature_hex,
        &payload.public_key_hex,
    ) {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub fn execute(
    psbt_hex: &str,
    redeem_script_hex: &str,
    input_index: usize,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<serde_json::Value> {
    let psbt_bytes = hex::decode(psbt_hex).context("Failed to decode PSBT hex")?;
    let mut psbt: Psbt = Psbt::deserialize(&psbt_bytes).context("Failed to deserialize PSBT")?;

    if input_index >= psbt.inputs.len() {
        return Err(anyhow::anyhow!(
            "Input index {} out of bounds (PSBT has {} inputs)",
            input_index,
            psbt.inputs.len()
        ));
    }

    // Decode and add the last signer's signature
    let public_key_bytes =
        hex::decode(public_key_hex).context("Failed to decode public key hex")?;
    let public_key = PublicKey::from_slice(&public_key_bytes).context("Invalid public key")?;

    let sig_bytes = hex::decode(signature_hex).context("Failed to decode signature hex")?;

    // Parse the signature (DER format + sighash byte)
    if sig_bytes.is_empty() {
        return Err(anyhow::anyhow!("Signature is empty"));
    }

    let sighash_byte = sig_bytes[sig_bytes.len() - 1];
    let der_sig = &sig_bytes[..sig_bytes.len() - 1];

    let signature = elements::secp256k1_zkp::ecdsa::Signature::from_der(der_sig)
        .context("Failed to parse DER signature")?;

    let bitcoin_sig = elements::bitcoin::ecdsa::Signature {
        signature,
        sighash_type: elements::bitcoin::EcdsaSighashType::from_consensus(sighash_byte as u32),
    };

    let redeem_script_bytes =
        hex::decode(redeem_script_hex).context("Failed to decode redeem script hex")?;
    let redeem_script = ScriptBuf::from_bytes(redeem_script_bytes);

    // Add the last signature and witness script to PSBT
    let input = &mut psbt.inputs[input_index];
    input.partial_sigs.insert(public_key, bitcoin_sig);

    if input.witness_script.is_none() {
        input.witness_script = Some(redeem_script);
    }

    let mut tx = psbt.clone().extract_tx_unchecked_fee_rate();

    // For each input, build the witness from partial signatures
    for (i, input) in psbt.inputs.iter().enumerate() {
        if input.partial_sigs.is_empty() {
            return Err(anyhow::anyhow!("Input {} has no partial signatures", i));
        }

        let witness_script = input
            .witness_script
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Input {} missing witness script", i))?;

        // For 2-of-2 multisig, build witness: OP_0 <sig1> <sig2> <redeemScript>
        // We need to order the signatures according to the public keys in the redeem script
        if input.partial_sigs.len() < 2 {
            return Err(anyhow::anyhow!(
                "Input {} requires 2 signatures but only has {}",
                i,
                input.partial_sigs.len()
            ));
        }

        // Extract public keys from the witness script (2-of-2 multisig format: OP_2 <pk1> <pk2> OP_2 OP_CHECKMULTISIG)
        let script_bytes = witness_script.as_bytes();
        let mut pubkeys = Vec::new();
        let mut i_byte = 0;
        while i_byte < script_bytes.len() {
            if script_bytes[i_byte] == 33 {
                // Compressed public key length
                if i_byte + 33 < script_bytes.len() {
                    let pk_bytes = &script_bytes[i_byte + 1..i_byte + 34];
                    if let Ok(pk) = PublicKey::from_slice(pk_bytes) {
                        pubkeys.push(pk);
                    }
                    i_byte += 34;
                } else {
                    break;
                }
            } else {
                i_byte += 1;
            }
        }

        if pubkeys.len() != 2 {
            return Err(anyhow::anyhow!(
                "Input {} witness script does not contain exactly 2 public keys",
                i
            ));
        }

        // Order signatures according to the public keys in the redeem script
        let sig1 = input
            .partial_sigs
            .get(&pubkeys[0])
            .ok_or_else(|| anyhow::anyhow!("Missing signature for first public key"))?;
        let sig2 = input
            .partial_sigs
            .get(&pubkeys[1])
            .ok_or_else(|| anyhow::anyhow!("Missing signature for second public key"))?;

        // Serialize signatures with sighash type
        let mut sig1_bytes = sig1.signature.serialize_der().to_vec();
        sig1_bytes.push(sig1.sighash_type.to_u32() as u8);

        let mut sig2_bytes = sig2.signature.serialize_der().to_vec();
        sig2_bytes.push(sig2.sighash_type.to_u32() as u8);

        tx.input[i].witness.push(vec![]); // OP_0 for multisig
        tx.input[i].witness.push(sig1_bytes);
        tx.input[i].witness.push(sig2_bytes);
        tx.input[i].witness.push(witness_script.to_bytes());
    }

    // Verify all inputs have witnesses
    let all_finalized = tx.input.iter().all(|input| !input.witness.is_empty());

    if !all_finalized {
        return Err(anyhow::anyhow!(
            "Not all inputs are finalized - transaction may be missing signatures"
        ));
    }

    let witnesses: Vec<_> = tx
        .input
        .iter()
        .map(|input| {
            input
                .witness
                .iter()
                .map(|w| hex::encode(w))
                .collect::<Vec<_>>()
        })
        .collect();

    let tx_bytes = elements::bitcoin::consensus::serialize(&tx);
    let tx_hex = hex::encode(&tx_bytes);

    let output = json!({
        "transaction_hex": tx_hex,
        "transaction_size": tx_bytes.len(),
        "witnesses": witnesses,
        "inputs": tx.input.len(),
        "outputs": tx.output.len(),
        "version": tx.version.0,
        "locktime": tx.lock_time.to_consensus_u32(),
    });

    Ok(output)
}
