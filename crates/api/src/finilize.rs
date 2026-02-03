use anyhow::{Context, Result};
use axum::Json;
use elements::{
    bitcoin::PublicKey,
    encode::{deserialize, serialize},
    pset::PartiallySignedTransaction,
    script::Script,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct FinalizeRequest {
    pub pset_hex: String,
    pub redeem_script_hex: String,
    pub input_index: usize,
    pub signature_hex: String,
    pub public_key_hex: String,
}

pub async fn finalize_handler(
    Json(payload): Json<FinalizeRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    match execute(
        &payload.pset_hex,
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
    pset_hex: &str,
    redeem_script_hex: &str,
    input_index: usize,
    signature_hex: &str,
    public_key_hex: &str,
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

    // Decode and add the last signer's signature
    let public_key_bytes =
        hex::decode(public_key_hex).context("Failed to decode public key hex")?;
    let public_key = PublicKey::from_slice(&public_key_bytes).context("Invalid public key")?;

    let sig_bytes = hex::decode(signature_hex).context("Failed to decode signature hex")?;

    let redeem_script_bytes =
        hex::decode(redeem_script_hex).context("Failed to decode redeem script hex")?;
    let redeem_script = Script::from(redeem_script_bytes);

    // Add the last signature and witness script to PSET
    let input = &mut pset.inputs_mut()[input_index];
    input.partial_sigs.insert(public_key, sig_bytes);

    if input.witness_script.is_none() {
        input.witness_script = Some(redeem_script);
    }

    let mut tx = pset
        .extract_tx()
        .context("Failed to extract transaction from PSET")?;

    // For each input, build the witness from partial signatures
    for (i, input) in pset.inputs().iter().enumerate() {
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
        //
        // TODO(ivanlele): I've spend too much time debugging coin flips here, need to refactor this properly later
        let sig1 = input
            .partial_sigs
            .get(&pubkeys[0])
            .ok_or_else(|| anyhow::anyhow!("Missing signature for first public key"))?;
        let sig2 = input
            .partial_sigs
            .get(&pubkeys[1])
            .ok_or_else(|| anyhow::anyhow!("Missing signature for second public key"))?;

        tx.input[i].witness.script_witness = vec![
            vec![], // OP_0 for multisig
            sig1.clone(),
            sig2.clone(),
            witness_script.to_bytes(),
        ];
    }

    // Verify all inputs have witnesses
    let all_finalized = tx
        .input
        .iter()
        .all(|input| !input.witness.script_witness.is_empty());

    if !all_finalized {
        return Err(anyhow::anyhow!(
            "Not all inputs are finalized - transaction may be missing signatures"
        ));
    }

    let witnesses: Vec<_> = tx
        .input
        .iter()
        .enumerate()
        .map(|(idx, input)| {
            json!({
                "input_index": idx,
                "witness_elements": input.witness.script_witness.len()
            })
        })
        .collect();

    let output = json!({
        "transaction_hex": hex::encode(serialize(&tx)),
        "txid": tx.txid().to_string(),
        "inputs": tx.input.len(),
        "outputs": tx.output.len(),
        "finalized": true,
        "witnesses": witnesses
    });

    Ok(output)
}
