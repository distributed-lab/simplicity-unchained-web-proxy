use axum::{Router, extract::Json, http::StatusCode, routing::post};
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use anyhow::{Context, Result, anyhow};

use base64::{self, Engine};
use elements::opcodes::all;
use elements::script::Builder;
use simplicityhl::{Arguments, CompiledProgram, WitnessValues};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub fn compile_hl_to_base64(hl_code: &str, include_debug: bool) -> Result<String, String> {
    let args = Arguments::default();

    let compiled = CompiledProgram::new(hl_code, args, include_debug)
        .map_err(|e| format!("compile error: {}", e))?;

    let satisfied = compiled
        .satisfy(WitnessValues::default())
        .map_err(|e| format!("satisfy error: {}", e))?;

    let (program_bytes, _witness_bytes) = satisfied.redeem().to_vec_with_witness();

    Ok(base64::engine::general_purpose::STANDARD.encode(&program_bytes))
}

fn is_data_token(s: &str) -> bool {
    if s.starts_with('<') {
        return true;
    }
    if s.starts_with("OP_") {
        return false;
    }
    s.len().is_multiple_of(2) && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn decode_hex_string(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    let clean_s = s.trim_matches(|c| c == '<' || c == '>');
    hex::decode(clean_s)
}

pub fn parse_human_readable(input: &str) -> Result<Vec<u8>> {
    let mut builder = Builder::new();

    for token in input.split_whitespace() {
        if token.starts_with("OP_PUSHBYTES_") {
            continue;
        }

        if token.starts_with("<") || is_data_token(token) {
            let bytes = decode_hex_string(token).context("User provided invalid hex")?;
            builder = builder.push_slice(&bytes);
            continue;
        }

        let op = match token {
            "OP_FALSE" | "OP_0" => all::OP_PUSHBYTES_0,
            "OP_TRUE" | "OP_1" => all::OP_PUSHNUM_1,
            "OP_PUSHNUM_NEG1" | "OP_1NEGATE" => elements::opcodes::all::OP_PUSHNUM_NEG1,
            "OP_PUSHNUM_1" => elements::opcodes::all::OP_PUSHNUM_1,
            "OP_PUSHNUM_2" | "OP_2" => elements::opcodes::all::OP_PUSHNUM_2,
            "OP_PUSHNUM_3" | "OP_3" => elements::opcodes::all::OP_PUSHNUM_3,
            "OP_PUSHNUM_4" | "OP_4" => elements::opcodes::all::OP_PUSHNUM_4,
            "OP_PUSHNUM_5" | "OP_5" => elements::opcodes::all::OP_PUSHNUM_5,
            "OP_PUSHNUM_6" | "OP_6" => elements::opcodes::all::OP_PUSHNUM_6,
            "OP_PUSHNUM_7" | "OP_7" => elements::opcodes::all::OP_PUSHNUM_7,
            "OP_PUSHNUM_8" | "OP_8" => elements::opcodes::all::OP_PUSHNUM_8,
            "OP_PUSHNUM_9" | "OP_9" => elements::opcodes::all::OP_PUSHNUM_9,
            "OP_PUSHNUM_10" | "OP_10" => elements::opcodes::all::OP_PUSHNUM_10,
            "OP_PUSHNUM_11" | "OP_11" => elements::opcodes::all::OP_PUSHNUM_11,
            "OP_PUSHNUM_12" | "OP_12" => elements::opcodes::all::OP_PUSHNUM_12,
            "OP_PUSHNUM_13" | "OP_13" => elements::opcodes::all::OP_PUSHNUM_13,
            "OP_PUSHNUM_14" | "OP_14" => elements::opcodes::all::OP_PUSHNUM_14,
            "OP_PUSHNUM_15" | "OP_15" => elements::opcodes::all::OP_PUSHNUM_15,
            "OP_PUSHNUM_16" | "OP_16" => elements::opcodes::all::OP_PUSHNUM_16,

            "OP_NOP" => elements::opcodes::all::OP_NOP,
            "OP_IF" => elements::opcodes::all::OP_IF,
            "OP_NOTIF" => elements::opcodes::all::OP_NOTIF,
            "OP_VERIFY" => elements::opcodes::all::OP_VERIFY,
            "OP_RETURN" => elements::opcodes::all::OP_RETURN,
            "OP_ELSE" => elements::opcodes::all::OP_ELSE,
            "OP_ENDIF" => elements::opcodes::all::OP_ENDIF,
            "OP_VERIF" => elements::opcodes::all::OP_VERIF,
            "OP_VERNOTIF" => elements::opcodes::all::OP_VERNOTIF,

            "OP_TOALTSTACK" => elements::opcodes::all::OP_TOALTSTACK,
            "OP_FROMALTSTACK" => elements::opcodes::all::OP_FROMALTSTACK,
            "OP_2DROP" => elements::opcodes::all::OP_2DROP,
            "OP_2DUP" => elements::opcodes::all::OP_2DUP,
            "OP_3DUP" => elements::opcodes::all::OP_3DUP,
            "OP_2OVER" => elements::opcodes::all::OP_2OVER,
            "OP_2ROT" => elements::opcodes::all::OP_2ROT,
            "OP_2SWAP" => elements::opcodes::all::OP_2SWAP,
            "OP_IFDUP" => elements::opcodes::all::OP_IFDUP,
            "OP_DEPTH" => elements::opcodes::all::OP_DEPTH,
            "OP_DROP" => elements::opcodes::all::OP_DROP,
            "OP_DUP" => elements::opcodes::all::OP_DUP,
            "OP_NIP" => elements::opcodes::all::OP_NIP,
            "OP_OVER" => elements::opcodes::all::OP_OVER,
            "OP_PICK" => elements::opcodes::all::OP_PICK,
            "OP_ROLL" => elements::opcodes::all::OP_ROLL,
            "OP_ROT" => elements::opcodes::all::OP_ROT,
            "OP_SWAP" => elements::opcodes::all::OP_SWAP,
            "OP_TUCK" => elements::opcodes::all::OP_TUCK,
            "OP_SIZE" => elements::opcodes::all::OP_SIZE,

            "OP_INVERT" => elements::opcodes::all::OP_INVERT,
            "OP_AND" => elements::opcodes::all::OP_AND,
            "OP_OR" => elements::opcodes::all::OP_OR,
            "OP_XOR" => elements::opcodes::all::OP_XOR,
            "OP_EQUAL" => elements::opcodes::all::OP_EQUAL,
            "OP_EQUALVERIFY" => elements::opcodes::all::OP_EQUALVERIFY,
            "OP_LSHIFT" => elements::opcodes::all::OP_LSHIFT,
            "OP_RSHIFT" => elements::opcodes::all::OP_RSHIFT,

            "OP_1ADD" => elements::opcodes::all::OP_1ADD,
            "OP_1SUB" => elements::opcodes::all::OP_1SUB,
            "OP_NEGATE" => elements::opcodes::all::OP_NEGATE,
            "OP_ABS" => elements::opcodes::all::OP_ABS,
            "OP_NOT" => elements::opcodes::all::OP_NOT,
            "OP_0NOTEQUAL" => elements::opcodes::all::OP_0NOTEQUAL,
            "OP_ADD" => elements::opcodes::all::OP_ADD,
            "OP_SUB" => elements::opcodes::all::OP_SUB,
            "OP_MUL" => elements::opcodes::all::OP_MUL,
            "OP_DIV" => elements::opcodes::all::OP_DIV,
            "OP_MOD" => elements::opcodes::all::OP_MOD,
            "OP_BOOLAND" => elements::opcodes::all::OP_BOOLAND,
            "OP_BOOLOR" => elements::opcodes::all::OP_BOOLOR,
            "OP_NUMEQUAL" => elements::opcodes::all::OP_NUMEQUAL,
            "OP_NUMEQUALVERIFY" => elements::opcodes::all::OP_NUMEQUALVERIFY,
            "OP_NUMNOTEQUAL" => elements::opcodes::all::OP_NUMNOTEQUAL,
            "OP_LESSTHAN" => elements::opcodes::all::OP_LESSTHAN,
            "OP_GREATERTHAN" => elements::opcodes::all::OP_GREATERTHAN,
            "OP_LESSTHANOREQUAL" => elements::opcodes::all::OP_LESSTHANOREQUAL,
            "OP_GREATERTHANOREQUAL" => elements::opcodes::all::OP_GREATERTHANOREQUAL,
            "OP_MIN" => elements::opcodes::all::OP_MIN,
            "OP_MAX" => elements::opcodes::all::OP_MAX,
            "OP_WITHIN" => elements::opcodes::all::OP_WITHIN,

            "OP_RIPEMD160" => elements::opcodes::all::OP_RIPEMD160,
            "OP_SHA1" => elements::opcodes::all::OP_SHA1,
            "OP_SHA256" => elements::opcodes::all::OP_SHA256,
            "OP_HASH160" => elements::opcodes::all::OP_HASH160,
            "OP_HASH256" => elements::opcodes::all::OP_HASH256,
            "OP_CODESEPARATOR" => elements::opcodes::all::OP_CODESEPARATOR,
            "OP_CHECKSIG" => elements::opcodes::all::OP_CHECKSIG,
            "OP_CHECKSIGVERIFY" => elements::opcodes::all::OP_CHECKSIGVERIFY,
            "OP_CHECKMULTISIG" => elements::opcodes::all::OP_CHECKMULTISIG,
            "OP_CHECKMULTISIGVERIFY" => elements::opcodes::all::OP_CHECKMULTISIGVERIFY,
            "OP_CHECKSIGFROMSTACK" => elements::opcodes::all::OP_CHECKSIGFROMSTACK,
            "OP_CHECKSIGFROMSTACKVERIFY" => elements::opcodes::all::OP_CHECKSIGFROMSTACKVERIFY,

            "OP_CAT" => elements::opcodes::all::OP_CAT,
            "OP_SUBSTR" => elements::opcodes::all::OP_SUBSTR,
            "OP_LEFT" => elements::opcodes::all::OP_LEFT,
            "OP_RIGHT" => elements::opcodes::all::OP_RIGHT,
            "OP_CLTV" => elements::opcodes::all::OP_CLTV,
            "OP_CSV" => elements::opcodes::all::OP_CSV,
            "OP_ADD64" => elements::opcodes::all::OP_ADD64,
            "OP_SUB64" => elements::opcodes::all::OP_SUB64,
            "OP_MUL64" => elements::opcodes::all::OP_MUL64,
            "OP_DIV64" => elements::opcodes::all::OP_DIV64,
            "OP_NEG64" => elements::opcodes::all::OP_NEG64,
            "OP_LESSTHAN64" => elements::opcodes::all::OP_LESSTHAN64,
            "OP_LESSTHANOREQUAL64" => elements::opcodes::all::OP_LESSTHANOREQUAL64,
            "OP_GREATERTHAN64" => elements::opcodes::all::OP_GREATERTHAN64,
            "OP_GREATERTHANOREQUAL64" => elements::opcodes::all::OP_GREATERTHANOREQUAL64,
            "OP_SCRIPTNUMTOLE64" => elements::opcodes::all::OP_SCRIPTNUMTOLE64,
            "OP_LE64TOSCRIPTNUM" => elements::opcodes::all::OP_LE64TOSCRIPTNUM,
            "OP_LE32TOLE64" => elements::opcodes::all::OP_LE32TOLE64,
            "OP_SHA256INITIALIZE" => elements::opcodes::all::OP_SHA256INITIALIZE,
            "OP_SHA256UPDATE" => elements::opcodes::all::OP_SHA256UPDATE,
            "OP_SHA256FINALIZE" => elements::opcodes::all::OP_SHA256FINALIZE,
            "OP_INSPECTINPUTOUTPOINT" => elements::opcodes::all::OP_INSPECTINPUTOUTPOINT,
            "OP_INSPECTINPUTASSET" => elements::opcodes::all::OP_INSPECTINPUTASSET,
            "OP_INSPECTINPUTVALUE" => elements::opcodes::all::OP_INSPECTINPUTVALUE,
            "OP_INSPECTINPUTSCRIPTPUBKEY" => elements::opcodes::all::OP_INSPECTINPUTSCRIPTPUBKEY,
            "OP_INSPECTINPUTSEQUENCE" => elements::opcodes::all::OP_INSPECTINPUTSEQUENCE,
            "OP_INSPECTINPUTISSUANCE" => elements::opcodes::all::OP_INSPECTINPUTISSUANCE,
            "OP_PUSHCURRENTINPUTINDEX" => elements::opcodes::all::OP_PUSHCURRENTINPUTINDEX,
            "OP_INSPECTOUTPUTASSET" => elements::opcodes::all::OP_INSPECTOUTPUTASSET,
            "OP_INSPECTOUTPUTVALUE" => elements::opcodes::all::OP_INSPECTOUTPUTVALUE,
            "OP_INSPECTOUTPUTNONCE" => elements::opcodes::all::OP_INSPECTOUTPUTNONCE,
            "OP_INSPECTOUTPUTSCRIPTPUBKEY" => elements::opcodes::all::OP_INSPECTOUTPUTSCRIPTPUBKEY,
            "OP_INSPECTVERSION" => elements::opcodes::all::OP_INSPECTVERSION,
            "OP_INSPECTLOCKTIME" => elements::opcodes::all::OP_INSPECTLOCKTIME,
            "OP_INSPECTNUMINPUTS" => elements::opcodes::all::OP_INSPECTNUMINPUTS,
            "OP_INSPECTNUMOUTPUTS" => elements::opcodes::all::OP_INSPECTNUMOUTPUTS,
            "OP_TXWEIGHT" => elements::opcodes::all::OP_TXWEIGHT,
            "OP_ECMULSCALARVERIFY" => elements::opcodes::all::OP_ECMULSCALARVERIFY,
            "OP_TWEAKVERIFY" => elements::opcodes::all::OP_TWEAKVERIFY,
            _ => return Err(anyhow!("unexpected behaviour")),
        };
        builder = builder.push_opcode(op);
    }

    Ok(builder.into_script().into_bytes())
}

#[derive(Deserialize)]
pub struct CompileRequest {
    pub script: String,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub hex: String,
}

pub async fn convert_handler(
    Json(script): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, String)> {
    let result = parse_human_readable(&script.script);

    match result {
        Ok(script) => Ok(Json(CompileResponse {
            hex: hex::encode(script),
        })),
        Err(_) => Err((StatusCode::BAD_REQUEST, "Invalid script".to_string())),
    }
}
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

pub async fn define_routes(config: Config) -> Result<()> {
    let router = Router::new()
        .route("/compile", post(compile_handler))
        .route("/convert", post(convert_handler));

    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str
        .parse()
        .context("Invalid address string in config")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

#[test]
fn test_elements_specific_opcode() -> Result<()> {
    let input = "OP_PUSHNUM_2 OP_CAT OP_PUSHBYTES_3 010203 OP_CHECKMULTISIG";

    let result = parse_human_readable(input)?;

    let expected_hex = "527e03010203ae";

    assert_eq!(hex::encode(result), expected_hex);
    Ok(())
}
#[test]
fn compile_hl_roundtrip_produces_same_program_bytes() {
    let hl = "fn main() { assert!(true); }";

    let b64 =
        compile_hl_to_base64(hl, /* include_debug = */ false).expect("compile_hl_to_base64 failed");

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .expect("base64 decode failed");

    let compiled = CompiledProgram::new(hl, Arguments::default(), /* include_debug = */ false)
        .expect("CompiledProgram::new failed");
    let satisfied = compiled
        .satisfy(WitnessValues::default())
        .expect("satisfy failed");
    let (program_bytes, _witness_bytes) = satisfied.redeem().to_vec_with_witness();

    assert_eq!(decoded, program_bytes, "compiled bytes differ");
}
