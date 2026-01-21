use std::collections::HashMap;

use anyhow::Result;
use axum::{extract::Json, http::StatusCode};
use serde::{Deserialize, Serialize};

use base64::engine::general_purpose::STANDARD;
use base64::{self, Engine};
use simplicityhl::parse::ParseFromStr;
use simplicityhl::str::WitnessName;
use simplicityhl::{ResolvedType, Value, WitnessValues};

#[derive(Debug, Deserialize)]
pub struct CompileRequestHl {
    /// SimplicityHL source text
    pub script: String,

    /// Optional: Witness for the program
    pub witness: Option<std::collections::HashMap<String, Witness>>,

    /// Optional: whether to include debug symbols in compilation
    #[serde(default)]
    pub include_debug: bool,
}

#[derive(Debug, Serialize)]
pub struct CompileResponseHl {
    /// Compiled Simplicity program (base64)
    pub program_base64: String,

    /// Optional: witness bytes (base64) if non-empty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Witness {
    pub value: String,
    #[serde(rename = "type")]
    pub type_: String,
}

pub async fn compile_handler(
    Json(req): Json<CompileRequestHl>,
) -> Result<Json<CompileResponseHl>, (StatusCode, String)> {
    let script = req.script;
    let include_debug = req.include_debug;

    let args = simplicityhl::Arguments::default();

    let compiled = simplicityhl::CompiledProgram::new(script, args, include_debug)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("compile error: {}", e)))?;

    let raw_witness = req.witness.unwrap_or_default();
    let mut converted_witness = HashMap::new();

    for (key, value) in raw_witness {
        let name = WitnessName::from_str_unchecked(key.as_str());
        let value = Value::parse_from_str(
            &value.value,
            &ResolvedType::parse_from_str(value.type_.as_str()).map_err(|e| {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("value of witness is incorrect: {}", e),
                );
            })?,
        )
        .map_err(|e| {
            return (
                StatusCode::BAD_REQUEST,
                format!("witness is incorrect: {}", e),
            );
        })?;

        converted_witness.insert(name, value);
    }

    let witness = WitnessValues::from(converted_witness);

    let satisfied = compiled
        .satisfy(witness)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("satisfy error: {}", e)))?;

    let (program_bytes, witness_bytes) = satisfied.redeem().to_vec_with_witness();

    let program_b64 = STANDARD.encode(&program_bytes);
    let witness_b64 = if witness_bytes.is_empty() {
        None
    } else {
        Some(STANDARD.encode(&witness_bytes))
    };

    Ok(Json(CompileResponseHl {
        program_base64: program_b64,
        witness_base64: witness_b64,
    }))
}

#[tokio::test]
async fn compile_hl_roundtrip_produces_same_program_bytes() {
    let script = "fn main() { assert!(true); }".to_string();

    let mut witness_map = std::collections::HashMap::new();
    witness_map.insert(
            "ALICE_SIGNATURE".to_string(),
            Witness {
                value: "0xf74b3ca574647f8595624b129324afa2f38b598a9c1c7cfc5f08a9c036ec5acd3c0fbb9ed3dae5ca23a0a65a34b5d6cccdd6ba248985d6041f7b21262b17af6f".to_string(),
                type_: "Signature".to_string(),
            }
        );

    let req = CompileRequestHl {
        script: script.clone(),
        witness: Some(witness_map),
        include_debug: false,
    };

    let result = compile_handler(Json(req))
        .await
        .expect("compile_handler failed");

    let Json(response) = result;

    let decoded_actual = base64::engine::general_purpose::STANDARD
        .decode(&response.program_base64)
        .expect("base64 decode failed");

    let compiled = simplicityhl::CompiledProgram::new(
        script,
        simplicityhl::Arguments::default(),
        /* include_debug = */ false,
    )
    .expect("CompiledProgram::new failed");

    let satisfied = compiled
        .satisfy(WitnessValues::default())
        .expect("satisfy failed");

    let (expected_program_bytes, _witness_bytes) = satisfied.redeem().to_vec_with_witness();

    assert_eq!(
        decoded_actual, expected_program_bytes,
        "handler output differs from direct compilation"
    );
}
