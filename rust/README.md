# BUTL Rust Reference Implementation

## `butl_v12.rs` - Complete Library for Building BUTL Applications in Rust

*From installing to sending your first BUTL-encrypted message in under 15 minutes.*

---

## What This Is

`butl_v12.rs` is a single-file Rust implementation of every feature in the BUTL Protocol v1.2:

- Self-sovereign identity (Bitcoin address from secp256k1 keypair)
- End-to-end encryption (ECDH + AES-256-GCM)
- Message integrity (SHA-256)
- Replay protection (Bitcoin block height freshness)
- Forward privacy (new address per message, cryptographically chained)
- Verify-before-download gate (8-step, two-phase)
- Proof of Satoshi (optional balance check, receiver-configurable toggle + slider)
- Pubkey-to-address consistency check

The implementation uses production-quality crates from the Rust ecosystem. All cryptographic operations use `secp256k1` (bindings to libsecp256k1, the same C library used by Bitcoin Core), `aes-gcm` (AEAD encryption), and `sha2` (SHA-256). There is no stub mode - this is real cryptography from the start.

---

## Quick Start

### Step 1: Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```bash
rustc --version
cargo --version
```

Rust 1.70 or newer is recommended.

### Step 2: Create a New Project

```bash
cargo new butl-demo
cd butl-demo
```

### Step 3: Add Dependencies

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "butl-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
secp256k1 = { version = "0.29", features = ["global-context", "rand-std"] }
sha2 = "0.10"
ripemd = "0.1"
aes-gcm = "0.10"
base64 = "0.22"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bs58 = { version = "0.5", features = ["check"] }
hex = "0.4"
```

| Crate | License | What It Does |
|-------|---------|--------------|
| `secp256k1` | CC0-1.0 | Keypair generation, ECDSA signing/verification, ECDH (wraps Bitcoin Core's libsecp256k1) |
| `sha2` | MIT OR Apache-2.0 | SHA-256 hashing |
| `ripemd` | MIT OR Apache-2.0 | RIPEMD-160 for Bitcoin address derivation |
| `aes-gcm` | MIT OR Apache-2.0 | AES-256-GCM authenticated encryption |
| `base64` | MIT OR Apache-2.0 | Base64 encoding for signatures and chain proofs |
| `rand` | MIT OR Apache-2.0 | Cryptographic random number generation |
| `serde` | MIT OR Apache-2.0 | Serialization framework |
| `serde_json` | MIT OR Apache-2.0 | JSON serialization for headers |
| `bs58` | MIT OR Apache-2.0 | Base58 encoding for Bitcoin addresses |
| `hex` | MIT OR Apache-2.0 | Hexadecimal encoding for public keys and hashes |

### Step 4: Copy the Source

Copy `butl_v12.rs` to `src/main.rs` in your project, replacing the generated file.

### Step 5: Build and Run

```bash
cargo run --release
```

The first build downloads and compiles all dependencies (takes 1-2 minutes). Subsequent builds are fast. The `--release` flag enables optimizations.

The demo runs five scenarios: genesis message, chained message, tampered payload detection, wrong receiver rejection, and Proof of Satoshi. Every step prints its verification status.

---

## Using the Library

### Create an Identity

```rust
use butl::{BUTLKeypair, sha256};

// Generate a random identity
let me = BUTLKeypair::generate().unwrap();

// Or from a specific seed (deterministic - same seed = same identity)
let me = BUTLKeypair::from_secret(&sha256(b"my-secret-seed")).unwrap();

// Your identity
println!("Address:    {}", me.address);            // 1A7bL3qK...
println!("Public Key: {}", me.public_key_hex);     // 02a1633c...
```

The private key is generated once, backed up immediately (ideally as a BIP-39 seed phrase), encrypted at rest, and never displayed or logged again. See [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md).

### Create a Keychain

A keychain generates a deterministic sequence of keypairs from a master seed. Each message uses a fresh keypair (address chaining).

```rust
use butl::BUTLKeychain;

let mut keychain = BUTLKeychain::new(sha256(b"my-master-seed"));

// First keypair (genesis)
let kp0 = keychain.next_keypair().unwrap();
println!("Message 0: {}", kp0.address);

// Second keypair (chained to first)
let kp1 = keychain.next_keypair().unwrap();
println!("Message 1: {}", kp1.address);

// Thread identity (constant across all messages)
println!("Thread ID: {}", keychain.thread_id());
```

### Send an Encrypted Message

```rust
use butl::{BUTLKeychain, BUTLSigner, StubBlockchain};

// Sender setup
let keychain = BUTLKeychain::new(sha256(b"sender-seed"));
let mut signer = BUTLSigner::new(keychain, StubBlockchain);

// Receiver's public identity (obtained via .well-known/butl.json,
// contact exchange, QR code, or any other method)
let rx_pubkey = receiver.public_key_bytes();  // [u8; 33]
let rx_addr = &receiver.address;               // &str

// Sign, encrypt, and package the message
let msg = signer.sign_and_encrypt(
    b"Hello from BUTL!",
    &rx_pubkey,
    rx_addr,
).unwrap();

// msg.header             - BUTL header (cleartext, for Phase 1)
// msg.encrypted_payload  - encrypted body (for Phase 2)
```

### Receive and Verify a Message

```rust
use butl::{BUTLKeypair, BUTLGate, StubBlockchain, ProofOfSatoshiConfig};

// Receiver setup
let receiver = BUTLKeypair::from_secret(&sha256(b"receiver-seed")).unwrap();
let gate = BUTLGate::new(
    receiver,
    StubBlockchain,
    144,                                // freshness window (~24 hours)
    ProofOfSatoshiConfig::default(),    // PoS disabled
);

// Phase 1: Verify header only (no payload on device yet)
let mut report = gate.check_header(&msg.header, None);

if report.gate_passed() {
    // Phase 2: Download and decrypt payload
    if let Some(plaintext) = gate.accept_payload(
        &msg.header,
        &msg.encrypted_payload,
        &mut report,
    ) {
        if report.fully_verified() {
            println!("Verified: {}", String::from_utf8_lossy(&plaintext));
        }
    }
} else {
    println!("Gate CLOSED - message rejected");
    println!("{}", report.summary());
}
```

### Chained Messages (Conversation)

```rust
// Sender sends message 1 (genesis)
let msg1 = signer.sign_and_encrypt(body1, &rx_pubkey, rx_addr).unwrap();
let s1pk = hex::decode(&msg1.header.sender_pubkey).unwrap();

// Receiver verifies message 1
let mut r1 = gate.check_header(&msg1.header, None);
let pt1 = gate.accept_payload(&msg1.header, &msg1.encrypted_payload, &mut r1);

// Sender sends message 2 (chained)
let msg2 = signer.sign_and_encrypt(body2, &rx_pubkey, rx_addr).unwrap();

// Receiver verifies message 2 - passes previous sender's pubkey
let mut r2 = gate.check_header(&msg2.header, Some(&s1pk));
let pt2 = gate.accept_payload(&msg2.header, &msg2.encrypted_payload, &mut r2);
```

The `prev_pk` parameter tells the gate which public key to verify the chain proof against. For genesis messages (no `prev_addr`), pass `None`.

### Proof of Satoshi (Optional)

```rust
use butl::ProofOfSatoshiConfig;

// Disabled (default) - pure math, no blockchain query
let gate_no_pos = BUTLGate::new(
    receiver.clone(), StubBlockchain, 144,
    ProofOfSatoshiConfig { enabled: false, min_satoshis: 1 },
);

// Enabled with threshold - sender must hold >= 10,000 satoshis
let gate_with_pos = BUTLGate::new(
    receiver.clone(), StubBlockchain, 144,
    ProofOfSatoshiConfig { enabled: true, min_satoshis: 10_000 },
);

// Enabled with high threshold - enterprise use
let gate_enterprise = BUTLGate::new(
    receiver.clone(), StubBlockchain, 144,
    ProofOfSatoshiConfig { enabled: true, min_satoshis: 100_000 },
);
```

When disabled, Step 5 is skipped. The protocol operates as pure math with zero external dependencies. When enabled, Step 5 queries the blockchain to verify the sender's balance.

### Header Serialization

```rust
// To text headers (email X-headers / HTTP headers)
let text = msg.header.to_text_headers();
println!("{}", text);
// BUTL-Version: 1
// BUTL-Sender: 1A7bL3qK...
// ...

// To JSON
let json = msg.header.to_json();

// To/from serde
let serialized = serde_json::to_string(&msg.header).unwrap();
let deserialized: BUTLHeader = serde_json::from_str(&serialized).unwrap();
```

### Reading the Gate Report

```rust
let mut report = gate.check_header(&header, None);

// Individual step results
println!("Structure:  {}", report.structural);   // PASS, FAIL, or SKIP
println!("Receiver:   {}", report.receiver_match);
println!("Signature:  {}", report.signature);
println!("Freshness:  {}", report.freshness);
println!("PoS:        {}", report.pos);
println!("Chain:      {}", report.chain);
println!("Hash:       {}", report.payload_hash);
println!("Decrypt:    {}", report.decryption);

// Aggregate results
println!("Gate open:     {}", report.gate_passed());
println!("Fully verified: {}", report.fully_verified());

// Human-readable summary
println!("{}", report.summary());

// Error details
for err in &report.errors {
    eprintln!("Error: {}", err);
}
```

### Custom Blockchain Provider

For production, implement the `BlockchainProvider` trait with a real Bitcoin data source:

```rust
use butl::BlockchainProvider;

struct MempoolBlockchain {
    client: reqwest::blocking::Client,
}

impl MempoolBlockchain {
    fn new() -> Self {
        Self { client: reqwest::blocking::Client::new() }
    }
}

impl BlockchainProvider for MempoolBlockchain {
    fn get_current_block(&self) -> (u64, String) {
        let height: u64 = self.client
            .get("https://mempool.space/api/blocks/tip/height")
            .send().unwrap().text().unwrap().parse().unwrap();
        let hash = self.client
            .get(format!("https://mempool.space/api/block-height/{}", height))
            .send().unwrap().text().unwrap();
        (height, hash)
    }

    fn verify_block(&self, height: u64, hash: &str) -> bool {
        let actual = self.client
            .get(format!("https://mempool.space/api/block-height/{}", height))
            .send().unwrap().text().unwrap();
        actual == hash
    }

    fn get_chain_height(&self) -> u64 {
        self.client
            .get("https://mempool.space/api/blocks/tip/height")
            .send().unwrap().text().unwrap().parse().unwrap()
    }

    fn check_balance(&self, address: &str, min_sats: u64) -> bool {
        let resp: serde_json::Value = self.client
            .get(format!("https://mempool.space/api/address/{}", address))
            .send().unwrap().json().unwrap();
        let funded = resp["chain_stats"]["funded_txo_sum"].as_u64().unwrap_or(0);
        let spent = resp["chain_stats"]["spent_txo_sum"].as_u64().unwrap_or(0);
        (funded - spent) >= min_sats
    }
}

// Use it
let blockchain = MempoolBlockchain::new();
let mut signer = BUTLSigner::new(keychain, blockchain);
```

This example uses `reqwest` (add `reqwest = { version = "0.11", features = ["blocking", "json"] }` to Cargo.toml). For async applications, use `reqwest` without the `blocking` feature.

---

## API Reference

### Types

| Type | Purpose |
|------|---------|
| `BUTLKeypair` | Single secp256k1 keypair. Identity, signing, ECDH, address derivation. |
| `BUTLKeychain` | Deterministic key sequence from master seed. Address chaining. |
| `BUTLSigner<B>` | Sender-side: sign, encrypt, package. Generic over `BlockchainProvider`. |
| `BUTLGate<B>` | Receiver-side: 8-step verify-before-download gate. Generic over `BlockchainProvider`. |
| `BUTLHeader` | Protocol header. Serde `Serialize` + `Deserialize`. Canonical payload. Text/JSON output. |
| `BUTLMessage` | Header + encrypted payload. The complete transmissible unit. |
| `ProofOfSatoshiConfig` | Toggle + slider for balance check. Receiver configuration. |
| `BlockchainProvider` | Trait: 4 methods for Bitcoin blockchain queries. |
| `StubBlockchain` | Dev/test stub returning hardcoded values. |
| `GateReport` | Detailed results of all 8 verification steps with `summary()`. |
| `CheckResult` | Enum: `Pass`, `Fail`, `Skip`. Implements `Display`. |
| `BUTLError` | Error enum: 6 variants covering all failure modes. |

### BUTLKeypair Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_secret` | `(bytes: &[u8; 32]) -> Result<Self>` | Create from 32-byte private key (deterministic) |
| `generate` | `() -> Result<Self>` | Create with random private key |
| `public_key_bytes` | `(&self) -> [u8; 33]` | Compressed public key as 33 bytes |
| `private_key_bytes` | `(&self) -> [u8; 32]` | Private key as 32 bytes (NEVER log or transmit) |
| `sign` | `(&self, hash: &[u8; 32]) -> Result<Vec<u8>>` | ECDSA sign. Returns DER-encoded signature. |
| `verify` | `(pubkey, sig, hash) -> Result<bool>` | Static. Verify ECDSA signature. |
| `ecdh` | `(&self, other: &[u8]) -> Result<[u8; 32]>` | Compute ECDH shared secret |

### BUTLKeychain Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(seed: [u8; 32]) -> Self` | Create from 32-byte master seed |
| `next_keypair` | `(&mut self) -> Result<&BUTLKeypair>` | Generate next keypair in chain |
| `current` | `(&self) -> Option<&BUTLKeypair>` | Current keypair |
| `previous` | `(&self) -> Option<&BUTLKeypair>` | Previous keypair (for chain proof) |
| `previous_address` | `(&self) -> String` | Previous address (for BUTL-PrevAddr) |
| `create_chain_proof` | `(&self, addr: &str) -> Result<Vec<u8>>` | Sign current address with previous key |
| `thread_id` | `(&self) -> String` | SHA-256 hex of first address |
| `seq_num` | `(&self) -> u32` | Current sequence number (0-indexed) |
| `history` | `(&self) -> &[String]` | All addresses generated |

### BUTLGate Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(rx, blockchain, window, pos_config) -> Self` | Create gate with configuration |
| `check_header` | `(&self, header, prev_pk) -> GateReport` | Phase 1: verify header (steps 1-6) |
| `accept_payload` | `(&self, header, payload, report) -> Option<Vec<u8>>` | Phase 2: verify and decrypt (steps 7-8) |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `VERSION` | `"1.2.0"` | Library version |
| `PROTOCOL_VERSION` | `1` | BUTL protocol version (wire format) |

---

## File Structure

```
rust/
|-- butl_v12.rs      Reference implementation (copy to src/main.rs)
|-- Cargo.toml       Cargo dependencies (copy to project root)
+-- README.md        This document
```

For the initial release, `butl_v12.rs` is a single file containing the full implementation and a demo `main()` function. Copy it to `src/main.rs` in any Cargo project.

For production deployment as a crate (planned for v1.3):

```bash
cargo add butl       # Not yet available - planned for crates.io
```

At that point, the library will be structured as a proper crate with `lib.rs`, separate modules, and the demo moved to `examples/`.

---

## Integration Examples

### Passwordless Website Login

```rust
// Server presents a challenge
let challenge = format!(
    r#"{{"nonce":"{}","block_height":{}}}"#,
    hex::encode(&random_nonce),
    blockchain.get_chain_height()
);

// Client signs the challenge with BUTL
let msg = signer.sign_and_encrypt(
    challenge.as_bytes(),
    &server_pubkey,
    &server_address,
).unwrap();

// Server verifies - if gate passes, the user is authenticated
let mut report = gate.check_header(&msg.header, None);
if report.gate_passed() {
    if let Some(_pt) = gate.accept_payload(
        &msg.header, &msg.encrypted_payload, &mut report
    ) {
        if report.fully_verified() {
            let user = &msg.header.sender;
            // Authenticated as user - no password needed
        }
    }
}
```

### Authenticated API Request

```rust
// Client builds and sends an authenticated request
let msg = signer.sign_and_encrypt(
    b"{\"action\": \"get_balance\", \"account\": \"12345\"}",
    &api_server_pubkey,
    &api_server_address,
).unwrap();

// Transmit: header as HTTP headers, payload as body
let mut request = reqwest::blocking::Client::new()
    .post("https://api.example.com/v1/data");

for line in msg.header.to_text_headers().lines() {
    if let Some((key, value)) = line.split_once(": ") {
        request = request.header(key, value);
    }
}

let response = request.body(msg.encrypted_payload).send().unwrap();
```

### Encrypted File Transfer

```rust
// Read and encrypt a file
let file_data = std::fs::read("document.pdf").unwrap();
let msg = signer.sign_and_encrypt(
    &file_data,
    &recipient_pubkey,
    &recipient_address,
).unwrap();

// Recipient verifies before saving to disk
let mut report = gate.check_header(&msg.header, None);
if report.gate_passed() {
    if let Some(plaintext) = gate.accept_payload(
        &msg.header, &msg.encrypted_payload, &mut report
    ) {
        if report.fully_verified() {
            std::fs::write("document_received.pdf", &plaintext).unwrap();
        }
    }
}
```

---

## Rust-Specific Notes

### Error Handling

Every fallible operation returns `Result<T, BUTLError>`. Use `?` for propagation or `match`/`unwrap()` for demos:

```rust
// With ? operator (recommended)
fn send_message(signer: &mut BUTLSigner<StubBlockchain>) -> Result<(), BUTLError> {
    let msg = signer.sign_and_encrypt(b"Hello", &rx_pk, &rx_addr)?;
    Ok(())
}

// With match (explicit error handling)
match signer.sign_and_encrypt(b"Hello", &rx_pk, &rx_addr) {
    Ok(msg) => println!("Sent to {}", msg.header.receiver),
    Err(BUTLError::Encryption(e)) => eprintln!("Encryption failed: {}", e),
    Err(BUTLError::Signing(e)) => eprintln!("Signing failed: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Generics Over BlockchainProvider

`BUTLSigner<B>` and `BUTLGate<B>` are generic over any type implementing the `BlockchainProvider` trait. This means you can swap `StubBlockchain` for a real provider without changing any other code:

```rust
// Development
let mut signer = BUTLSigner::new(keychain, StubBlockchain);

// Production (same API, real blockchain)
let mut signer = BUTLSigner::new(keychain, MempoolBlockchain::new());
```

### Clone and Ownership

`BUTLKeypair` implements `Clone`. When you need to retain a keypair reference across operations (e.g., saving the sender's pubkey from message 1 to verify message 2's chain proof), clone the relevant data:

```rust
let s1pk = hex::decode(&msg1.header.sender_pubkey).unwrap();
// s1pk is now owned, msg1 can be dropped
let mut r2 = gate.check_header(&msg2.header, Some(&s1pk));
```

### Serde Integration

`BUTLHeader` derives `Serialize` and `Deserialize`. It works with any serde-compatible format:

```rust
// JSON
let json = serde_json::to_string_pretty(&header).unwrap();
let parsed: BUTLHeader = serde_json::from_str(&json).unwrap();

// MessagePack (add rmp-serde to Cargo.toml)
let bytes = rmp_serde::to_vec(&header).unwrap();
let parsed: BUTLHeader = rmp_serde::from_slice(&bytes).unwrap();

// CBOR, TOML, YAML - any serde format works
```

### Memory Safety

Rust's ownership model prevents many classes of memory bugs, but private key handling still requires care:

- Private keys exist as `SecretKey` inside `BUTLKeypair`. When the keypair is dropped, Rust's `Drop` implementation for `SecretKey` zeros the memory.
- For explicit zeroing before drop, use the `zeroize` crate (add `zeroize = "1.7"` to Cargo.toml).
- Never convert private keys to `String` - strings may be copied, logged, or interned by the runtime.

---

## Security Notes

- **Real cryptography from the start.** Unlike the Python implementation, there is no stub mode. The Rust implementation uses production-quality libraries for all operations.
- **libsecp256k1.** The `secp256k1` crate wraps the same C library used by Bitcoin Core. This is constant-time, battle-tested code that secures the Bitcoin network.
- **Blockchain stub.** `StubBlockchain` returns hardcoded values. Production deployments must implement `BlockchainProvider` with a real Bitcoin data source.
- **Key storage.** This library does not handle key storage. Private keys must be encrypted at rest. See [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md).
- **No unsafe code.** The implementation uses no `unsafe` blocks (the `secp256k1` crate uses `unsafe` internally for FFI, but that is outside this codebase).

---

## Troubleshooting

### "error: linker `cc` not found"

The `secp256k1` crate requires a C compiler to build libsecp256k1. Install one:

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf install gcc
```

### "error[E0433]: failed to resolve: use of undeclared crate"

Dependencies are missing from `Cargo.toml`. Make sure all 10 crates are listed in `[dependencies]`. Run `cargo build` again after updating.

### "secp256k1: secret key must be 32 bytes"

The input to `BUTLKeypair::from_secret()` must be exactly 32 bytes. Use `sha256()` to derive a 32-byte key from any seed:

```rust
let kp = BUTLKeypair::from_secret(&sha256(b"my-seed")).unwrap();
```

### "GCM auth failed - tampered or wrong key"

The ECDH shared secret doesn't match. This is expected when the wrong receiver tries to decrypt (different private key = different shared secret), the payload was modified in transit, or the AAD doesn't match (addresses changed between signing and verification).

---

## Related Documents

| Document | Description |
|----------|-------------|
| [BUTL_Protocol_Specification_v1.2.md](../spec/BUTL_Protocol_Specification_v1.2.md) | Complete protocol specification |
| [HEADER_REGISTRY.md](../spec/HEADER_REGISTRY.md) | All BUTL header fields defined |
| [TEST_VECTORS.md](../spec/TEST_VECTORS.md) | Deterministic test cases for interoperability |
| [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md) | Critical key safety information |
| [butl_mvp_v12.py](../mvp/butl_mvp_v12.py) | Zero-dependency Python MVP proving all 8 claims |
| [butl_v12.py](../python/butl_v12.py) | Python reference implementation |
| [SECURITY.md](../SECURITY.md) | Security policy and known limitations |

---

*One file. Ten crates. Real secp256k1. Eight-step verified trust. Get started in under 15 minutes.*
