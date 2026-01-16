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

**Endpoint:** `POST /convert`

#### Request
```bash
curl -X POST http://localhost:3000/convert \
   -H "Content-Type: application/json" \
   -d '{
     "script": "OP_PUSHNUM_2 OP_CAT OP_CHECKMULTISIG"
   }'
```

#### Response
```json
{
  "hex": "527eae"
}
```

### 2. CLI (Command Line Interface)

The application is launched via the command line, allowing configuration of the server's listening address.

- **Usage:**
  ```bash
  cargo run -- [OPTIONS]
