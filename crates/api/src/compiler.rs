use anyhow::Result;
use axum::{extract::Json, http::StatusCode};
use hal_simplicity::hal_simplicity::Program;
use serde::{Deserialize, Serialize};
use simplicity::jet::Elements;

use base64::engine::general_purpose::STANDARD;
use base64::{self, Engine};

#[derive(Debug, Deserialize)]
pub struct CompileRequestHl {
    /// SimplicityHL source text
    pub script: String,

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

    pub cmr: String,
}

pub async fn compile_handler(
    Json(req): Json<CompileRequestHl>,
) -> Result<Json<CompileResponseHl>, (StatusCode, String)> {
    let script = req.script;
    let include_debug = req.include_debug;

    let args = simplicityhl::Arguments::default();

    let compiled = simplicityhl::CompiledProgram::new(script, args, include_debug)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("compile error: {}", e)))?;

    let satisfied = compiled
        .satisfy(simplicityhl::WitnessValues::default())
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("satisfy error: {}", e)))?;

    let (program_bytes, witness_bytes) = satisfied.redeem().to_vec_with_witness();

    let program =
        Program::<Elements>::from_bytes(&program_bytes, Some(&witness_bytes)).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("program parse error: {}", e),
            )
        })?;

    let program_b64 = STANDARD.encode(&program_bytes);
    let witness_b64 = if witness_bytes.is_empty() {
        None
    } else {
        Some(STANDARD.encode(&witness_bytes))
    };
    let cmr_bytes = program.commit_prog().cmr().to_byte_array();
    let cmr = hex::encode(cmr_bytes);

    Ok(Json(CompileResponseHl {
        program_base64: program_b64,
        witness_base64: witness_b64,
        cmr,
    }))
}

#[tokio::test]
async fn compile_hl_roundtrip_produces_same_program_bytes() {
    let script = "fn main() { assert!(true); }".to_string();
    let req = CompileRequestHl {
        script: script.clone(),
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
        .satisfy(simplicityhl::WitnessValues::default())
        .expect("satisfy failed");

    let (expected_program_bytes, _witness_bytes) = satisfied.redeem().to_vec_with_witness();

    assert_eq!(
        decoded_actual, expected_program_bytes,
        "handler output differs from direct compilation"
    );
}
