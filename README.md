# Simplicity Unchained Web Proxy

This service provides an HTTP API to compile and convert Elements and Simplicity scripts. It is designed to be used alongside other Simplicity tools or as a standalone compiler service.

## Interfaces

The service exposes interactions primarily through a RESTful HTTP API and is configured via a Command Line Interface (CLI).

## API Usage

### Compile Script
Compiles a source script into a base64 encoded program.

**Endpoint:** `POST /compile`

#### Request
```bash
curl -X POST http://localhost:3000/compile \
  -H "Content-Type: application/json" \
  -d '{
    "script": "fn main() { assert!(true); }",
    "include_debug": false
  }'
```

#### Response
```json
{
  "program_base64": "0pkEYBAmKDgU"
}
```

---

### Convert Bitcoin Script
Converts human-readable Bitcoin Script opcodes into a hex string.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/convert`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/convert \
   -H "Content-Type: application/json" \
   -d '{
     "script": "OP_PUSHNUM_2 OP_CAT OP_CHECKMULTISIG"
   }'
```

#### Response
```json
{
  "hex": "527eae",
  "address": "tlq1qq2g07nju42l0nlx0erqa3wsel2l8prnq96rlnhml262mcj7pe8w6ndvvyg237japt83z24m8gu4v3yfhaqvrqxydadc9scsmw"
}
```

---

### Create PSET
Constructs an unsigned Partially Signed Elements Transaction (PSET). This endpoint automatically fetches the required Witness UTXO data for the inputs from the Blockstream API.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/create-pset`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/create-pset \
   -H "Content-Type: application/json" \
   -d '{
     "inputs": ["7c2a9d8705574505357303861502444585f393153753b73eb49e852d89233503:0"],
     "outputs": [
       "tlq1qq2g07nju42l0nlx0erqa3wsel2l8prnq96rlnhml262mcj7pe8w6ndvvyg237japt83z24m8gu4v3yfhaqvrqxydadc9scsmw:99000",
       "fee:1000"
     ],
     "network": "liquid_testnet",
     "asset_id": null
   }'
   
```
   
#### Response
```json
{
  "pset": "cHNldP8BAgQCAAAAAAQAAAAAAAAAAAAAA...",
  "inputs": 1,
  "outputs": 2,
  "network": "liquid_testnet",
  "asset": "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49"
}
```

---
### Sign PSET
Signs a PSET input with a private key. This endpoint calculates the SegWit v0 sighash for the specified input, generates an ECDSA signature, and attaches it to the PSET.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/sign-pset`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/sign-pset \
   -H "Content-Type: application/json" \
   -d '{
     "pset_hex": "cHNldP8BAgQCAAAAAAQAAAAAAAAAAAAAA...",
     "secret_key_hex": "804622cda0d8e634317a12651d91751ceff5c081f2b5f63ef7912725c7275e5d",
     "input_index": 0,
     "redeem_script_hex": "522103..."
   }'
```

#### Response
```json
{
  "pset": "cHNldP8BAgQCAAAAAAQAAAAAAAAAAAAAA...",
  "signature_hex": "30440220...",
  "public_key_hex": "033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf",
  "input_index": 0,
  "partial_sigs_count": 1
}
```

---

### Finalize PSET
Takes a fully signed PSET, validates the signatures, and constructs the final witness data (specifically for 2-of-2 multisig inputs). It extracts the raw, broadcast-ready transaction hex.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/finalize`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/finalize \
   -H "Content-Type: application/json" \
   -d '{
     "pset_hex": "cHNldP8BAgQCAAAAAAQAAAAAAAAAAAAAA..."
   }'
```

#### Response
```json
{
  "transaction_hex": "02000000000101...",
  "txid": "7c2a9d8705574505357303861502444585f393153753b73eb49e852d89233503",
  "inputs": 1,
  "outputs": 2,
  "finalized": true,
  "witnesses": [
    {
      "input_index": 0,
      "witness_elements": 4
    }
  ]
}
```

### 2. CLI (Command Line Interface)

The application is launched via the command line, allowing configuration of the server's listening address.

- **Usage:**
  ```bash
  cargo run -- [OPTIONS]
