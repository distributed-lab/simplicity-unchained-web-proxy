# Simplicity Unchained Web Proxy

This service provides an HTTP API to compile and convert Elements and Simplicity scripts. It is designed to be used alongside other Simplicity tools or as a standalone compiler service.

## Interfaces

The service exposes interactions primarily through a RESTful HTTP API and is configured via a Command Line Interface (CLI).

## API Usage

### Compile Script
Compiles a source script into a base64 encoded program.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/compile`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/compile \
  -H "Content-Type: application/json" \
  -d '{
    "script": "fn main() { assert!(true); }",
    "include_debug": false
  }'
```

#### Response
```json
{
  "program_base64": "0pkEYBAmKDgU",
  "cmr":"a61f806cc2ec60a7d668fa3bac0e3e6e935700ddd690c9c985e5fac1afa2f130"
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
     "script": "OP_PUSHNUM_2 OP_CAT OP_CHECKMULTISIG",
     "network": "liquid_testnet"
   }'
```

#### Response
```json
{
  "hex":"527eae",
  "address":"tex1q6fqu642lyhu2g6ly0jzlsx36h5erh3f4y2hl9qv3eu6nspjhypjq5hlq4x"
}
```

---

### Create PSET
Constructs an unsigned Partially Signed Elements Transaction (PSET). This endpoint automatically fetches the required Witness UTXO data for the inputs from the Blockstream API.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/create-pset`

#### Request
```bash
curl -s -X POST "http://localhost:3000/simplicity-unchained-web-proxy-demo/create-pset" \
  -H "Content-Type: application/json" \
  -d '{
  "inputs": [
    "f539972050acd5c3020ff9d366af334e509160568b787121c5a6d58d3922a145:0"
  ],
  "outputs": [
    "tex1qg3uhk0r2knr2j6p85xqyd6nr2udpxe5t9jxusj5z54w67md0emnq229qdq:99000",
    "fee:1000"
  ],
  "asset_id": null,
  "network": "liquid_testnet"
}'
   
```
   
#### Response
```json
{
  "asset":"144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49",
  "inputs":1,
  "network":"liquid_testnet",
  "outputs":2,
  "pset":"70736574ff0102040200000001030400000000010401010105010201fb04020000000001014e01499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000186a00022002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee601070001080100010e2045a122398dd5a6c52171788b566091504e33af66d3f90f02c3d5ac50209739f5010f0400000000011004ffffffff00010308b88201000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c14010422002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee600010308e80300000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c1401040000"}
```

---
### Sign PSET
Signs a PSET input with a private key. This endpoint calculates the SegWit v0 sighash for the specified input, generates an ECDSA signature, and attaches it to the PSET.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/sign-pset`

#### Request
```bash
curl -s -X POST "http://localhost:3000/simplicity-unchained-web-proxy-demo/sign-pset" \
  -H "Content-Type: application/json" \
  -d '{
  "pset_hex": "70736574ff0102040200000001030400000000010401010105010201fb04020000000001014e01499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000186a00022002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee601070001080100010e2045a122398dd5a6c52171788b566091504e33af66d3f90f02c3d5ac50209739f5010f0400000000011004ffffffff00010308b88201000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c14010422002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee600010308e80300000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c1401040000",
  "secret_key_hex": "804622cda0d8e634317a12651d91751ceff5c081f2b5f63ef7912725c7275e5d",
  "input_index": 0,
  "redeem_script_hex": "5221033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf21034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa52ae"
}'
```

#### Response
```json
{
  "input_index":0,
  "partial_sigs_count":1,
  "pset":"70736574ff0102040200000001030400000000010401010105010201fb04020000000001014e01499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000186a00022002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee62202033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf473044022030535235414d40273b6346e3ec140112ad1abd8e8dfdda45830f548c453a213b02205696cf8dd689c0789e10a48642e48da6cc8671ee4788f8ea0623960bc7204080010105475221033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf21034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa52ae01070001080100010e2045a122398dd5a6c52171788b566091504e33af66d3f90f02c3d5ac50209739f5010f0400000000011004ffffffff00010308b88201000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c14010422002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee600010308e80300000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c1401040000",
  "public_key_hex":"033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf",
  "signature_hex":"3044022030535235414d40273b6346e3ec140112ad1abd8e8dfdda45830f548c453a213b02205696cf8dd689c0789e10a48642e48da6cc8671ee4788f8ea0623960bc720408001"}
```

---

### Finalize PSET
Takes a fully signed PSET, validates the signatures, and constructs the final witness data (specifically for 2-of-2 multisig inputs). It extracts the raw, broadcast-ready transaction hex.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/finalize`

#### Request
```bash
curl -s -X POST "http://localhost:3000/simplicity-unchained-web-proxy-demo/finalize" \
  -H "Content-Type: application/json" \
  -d '{
  "pset_hex": "70736574ff0102040200000001030400000000010401010105010201fb04020000000001014e01499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000186a00022002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee62202033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf473044022030535235414d40273b6346e3ec140112ad1abd8e8dfdda45830f548c453a213b02205696cf8dd689c0789e10a48642e48da6cc8671ee4788f8ea0623960bc7204080012202034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa47304402207f6a25942703dfe84eb7ca826d40f2cdd3726ae54281e7d035ef0524bf37b545022074404b9cd4230c2483195670e4705c628e04d9132e6f1c403304dd8481d45a9e010105475221033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf21034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa52ae01070001080100010e2045a122398dd5a6c52171788b566091504e33af66d3f90f02c3d5ac50209739f5010f0400000000011004ffffffff00010308b88201000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c14010422002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee600010308e80300000000000007fc04707365740220499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c1401040000"
}'
```

#### Response
```json
{
  "finalized":true,
  "inputs":1,
  "outputs":2,
  "transaction_hex":"02000000010145a122398dd5a6c52171788b566091504e33af66d3f90f02c3d5ac50209739f50000000000ffffffff0201499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000182b80022002044797b3c6ab4c6a96827a18046ea63571a13668b2c8dc84a82a55daf6dafcee601499a818545f6bae39fc03b637f2a4e1e64e590cac1bc3a6f6d71aa4443654c140100000000000003e800000000000000000400473044022030535235414d40273b6346e3ec140112ad1abd8e8dfdda45830f548c453a213b02205696cf8dd689c0789e10a48642e48da6cc8671ee4788f8ea0623960bc72040800147304402207f6a25942703dfe84eb7ca826d40f2cdd3726ae54281e7d035ef0524bf37b545022074404b9cd4230c2483195670e4705c628e04d9132e6f1c403304dd8481d45a9e01475221033523982d58e94be3b735731593f8225043880d53727235b566c515d24a0f7baf21034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa52ae0000000000",
  "txid":"3d9e856b8d53fa3c0507d70f4007364667cbf5fdd11b55e13cae040480171d85",
  "witnesses":[
    {
      "input_index":0,
      "witness_elements":4
    }
  ]
}
```

---

### Generate Keypair
Generates a random secp256k1 keypair (Secret Key and Compressed Public Key).

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/generate`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/generate \
   -H "Content-Type: application/json" \
   -d '{}'
```

### Response
```
{
  "compressed": true,
  "public_key": "03db8804e872ee79bb2b8b014f60530c448eaa108cb2582794a817e9d30aaf21ff",
  "secret_key": "777427aa93677f5e3e31edc8137884162593bb796e6eb84f99fb0c92dbb8e993"
}
```

---

### Sign message
Signs a received hex.

**Endpoint:** `POST /simplicity-unchained-web-proxy-demo/sign_message`

#### Request
```bash
curl -X POST http://localhost:3000/simplicity-unchained-web-proxy-demo/sign_message \
  -H "Content-Type: application/json" \
  -d '{
    "digest_hex": "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
    "secret_key_hex": "0000000000000000000000000000000000000000000000000000000000000001"
  }'
```

#### Response
```json
{
  "digest_hex":"5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
  "public_key_hex":"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
  "signature_hex":"3045022100b9bd37c202d27f5b3aac350de8020c0bd1dd9b7fae368e344b860ae7be853dac02203e6f312443f9eabd3bd51861af7e31b931d7e58412f740e1328af60cd72908e8"
}  
```

### 2. CLI (Command Line Interface)

The application is launched via the command line, allowing configuration of the server's listening address.

- **Usage:**
  ```bash
  cargo run -- [OPTIONS]
