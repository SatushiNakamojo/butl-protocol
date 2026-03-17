# BUTL Protocol - AI Code Generation Reference

## Exact Algorithms, Byte Formats, and Data Structures

**Purpose:** This document provides everything an AI needs to generate a correct, compatible BUTL implementation in any programming language. Every algorithm is specified at the byte level. Every format is exact. Every computation is deterministic. An AI that follows this document will produce code that passes the [test vectors](../spec/TEST_VECTORS.md) and interoperates with the Python and Rust reference implementations.

**When to use this:** When asking an AI to write BUTL code - a full implementation, a single function, a specific algorithm, or a port to a new language.

---

## 1. Constants

### secp256k1 Curve Parameters

```
p  = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
n  = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
Gx = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
Gy = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
a  = 0
b  = 7
```

Curve equation: `y^2 = x^3 + 7 (mod p)`

### Protocol Constants

```
PROTOCOL_VERSION       = 1
DEFAULT_FRESHNESS      = 144        (blocks, ~24 hours)
AES_GCM_IV_SIZE        = 12         (bytes)
AES_GCM_TAG_SIZE       = 16         (bytes)
AES_GCM_KEY_SIZE       = 32         (bytes)
SHA256_OUTPUT_SIZE     = 32         (bytes)
COMPRESSED_PUBKEY_SIZE = 33         (bytes)
PRIVATE_KEY_SIZE       = 32         (bytes)
ADDRESS_VERSION_BYTE   = 0x00       (P2PKH mainnet)
```

---

## 2. Key Generation

### From Seed to Keypair

```
INPUT:  seed (32 bytes)

STEP 1: Private key derivation
  private_key_int = int(seed) mod (n - 1) + 1
  // Ensures private key is in valid range [1, n-1]

STEP 2: Public key derivation (secp256k1 scalar multiplication)
  public_key_point = private_key_int x G
  // G is the generator point (Gx, Gy)
  // Result is a point (x, y) on the curve

STEP 3: Compress public key
  prefix = 0x02 if (y % 2 == 0) else 0x03
  compressed_pubkey = prefix || x.to_bytes(32, big_endian)
  // Result: 33 bytes (1 byte prefix + 32 bytes x-coordinate)

OUTPUT: private_key (32 bytes), compressed_pubkey (33 bytes)
```

### From Public Key to Bitcoin Address

```
INPUT:  compressed_pubkey (33 bytes)

STEP 1: SHA-256
  sha_hash = SHA-256(compressed_pubkey)
  // 32 bytes

STEP 2: RIPEMD-160
  ripemd_hash = RIPEMD-160(sha_hash)
  // 20 bytes

STEP 3: Add version byte
  versioned = 0x00 || ripemd_hash
  // 21 bytes (version byte + 20-byte hash)

STEP 4: Checksum
  checksum = SHA-256(SHA-256(versioned))[0:4]
  // First 4 bytes of double SHA-256

STEP 5: Base58Check encode
  full = versioned || checksum
  // 25 bytes
  address = Base58Encode(full)
  // String, 25-34 characters, starts with "1"

OUTPUT: address (string)
```

### Base58 Encoding

```
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
// 58 characters. Excludes: 0, O, I, l (to avoid visual ambiguity)

ALGORITHM:
  n = bytes_to_big_integer(input)
  result = ""
  while n > 0:
    n, remainder = divmod(n, 58)
    result = ALPHABET[remainder] + result

  // Preserve leading zero bytes as "1" characters
  for each byte in input:
    if byte == 0x00:
      result = "1" + result
    else:
      break

  return result
```

### Pubkey-to-Address Consistency Check

```
INPUT:  claimed_address (string), pubkey_hex (66 hex chars)

STEP 1: Decode pubkey
  pubkey_bytes = hex_decode(pubkey_hex)
  // 33 bytes

STEP 2: Derive address from pubkey
  derived_address = pubkey_to_address(pubkey_bytes)
  // Using the algorithm above

STEP 3: Compare
  consistent = (derived_address == claimed_address)

OUTPUT: consistent (boolean)
```

---

## 3. Keychain (Address Chaining)

### Key Derivation from Seed + Index

```
INPUT:  seed (32 bytes), index (unsigned 32-bit integer)

STEP 1: Concatenate
  data = seed || index.to_bytes(4, big_endian)
  // 36 bytes

STEP 2: Hash
  private_key = SHA-256(data)
  // 32 bytes

STEP 3: Generate keypair
  keypair = keygen(private_key)
  // Using key generation algorithm from Section 2

OUTPUT: keypair (private_key, compressed_pubkey, address)
```

### Chain Proof Construction

```
INPUT:  current_address (string), previous_private_key (32 bytes)

STEP 1: Hash the current address
  chain_hash = SHA-256(UTF-8(current_address))
  // 32 bytes

STEP 2: Sign with previous key
  chain_proof = ECDSA_SIGN(previous_private_key, chain_hash)
  // DER-encoded signature

OUTPUT: chain_proof (DER bytes)
```

### Chain Proof Verification

```
INPUT:  current_address (string), chain_proof (DER bytes),
        previous_pubkey (33 bytes compressed)

STEP 1: Hash the current address
  chain_hash = SHA-256(UTF-8(current_address))
  // 32 bytes

STEP 2: Verify with previous pubkey
  valid = ECDSA_VERIFY(previous_pubkey, chain_proof, chain_hash)

OUTPUT: valid (boolean)
```

### Thread ID

```
INPUT:  first_address_in_chain (string)

thread_id = hex_encode(SHA-256(UTF-8(first_address_in_chain)))
// 64 lowercase hex characters

OUTPUT: thread_id (string, 64 hex chars)
```

---

## 4. Canonical Signing Payload

### Construction

```
INPUT:  header fields (see below)

payload = "BUTL-Version:{version}\n"
        + "BUTL-Sender:{sender_address}\n"
        + "BUTL-SenderPubKey:{sender_pubkey_hex}\n"
        + "BUTL-Receiver:{receiver_address}\n"
        + "BUTL-ReceiverPubKey:{receiver_pubkey_hex}\n"
        + "BUTL-BlockHeight:{block_height}\n"
        + "BUTL-BlockHash:{block_hash}\n"
        + "BUTL-PayloadHash:{payload_hash}\n"
        + "BUTL-PrevAddr:{prev_addr_or_empty}\n"
        + "BUTL-Nonce:{nonce_or_empty}"

// CRITICAL RULES:
// - Separator is \n (0x0A), NOT \r\n
// - No spaces around colons
// - Field order is FIXED (exactly as shown above)
// - Empty string for absent optional values (field present, value empty)
// - All 10 lines present even if value is empty
// - No trailing newline after the last field
```

### Signing

```
INPUT:  canonical_payload (string), sender_private_key (32 bytes)

STEP 1: Hash the payload
  message_hash = SHA-256(UTF-8(canonical_payload))
  // 32 bytes

STEP 2: Sign
  signature = ECDSA_SIGN(sender_private_key, message_hash)
  // DER-encoded

STEP 3: Encode for header
  signature_b64 = Base64_Encode(signature)

OUTPUT: signature_b64 (string)
```

### Verification

```
INPUT:  header (all fields), signature_b64 (string),
        sender_pubkey (33 bytes compressed)

STEP 1: Reconstruct canonical payload from header fields
  // Same construction as above, using values from the header

STEP 2: Hash
  message_hash = SHA-256(UTF-8(canonical_payload))

STEP 3: Decode signature
  signature_der = Base64_Decode(signature_b64)

STEP 4: Verify
  valid = ECDSA_VERIFY(sender_pubkey, signature_der, message_hash)

OUTPUT: valid (boolean)
```

---

## 5. ECDH Key Agreement

### Shared Secret Computation

```
INPUT:  my_private_key (32 bytes), their_compressed_pubkey (33 bytes)

STEP 1: Decompress their public key
  their_point = decompress(their_compressed_pubkey)
  // (x, y) point on secp256k1

STEP 2: Scalar multiplication
  shared_point = my_private_key x their_point
  // Elliptic curve scalar multiplication

STEP 3: Hash the x-coordinate
  shared_secret = SHA-256(shared_point.x.to_bytes(32, big_endian))
  // 32 bytes

OUTPUT: shared_secret (32 bytes)
```

### Decompression Algorithm

```
INPUT:  compressed (33 bytes: prefix || x_bytes)

STEP 1: Extract
  prefix = compressed[0]    // 0x02 or 0x03
  x = bytes_to_big_integer(compressed[1:33])

STEP 2: Compute y
  y_squared = (x^3 + 7) mod p
  y = modular_sqrt(y_squared, p)
  // y = y_squared^((p+1)/4) mod p  (works because p = 3 mod 4)

STEP 3: Adjust parity
  if (y % 2 == 0) and (prefix == 0x03):
    y = p - y
  if (y % 2 == 1) and (prefix == 0x02):
    y = p - y

OUTPUT: (x, y) point on secp256k1
```

### Commutativity Property (MUST hold)

```
// Sender computes:
sender_secret = SHA-256( (sender_priv x receiver_pub).x )

// Receiver computes:
receiver_secret = SHA-256( (receiver_priv x sender_pub).x )

// These MUST be identical:
assert sender_secret == receiver_secret
```

---

## 6. Payload Encryption

### Encrypt

```
INPUT:  plaintext (bytes), shared_secret (32 bytes),
        sender_address (string), receiver_address (string)

STEP 1: Generate random IV
  iv = random_bytes(12)
  // 12 bytes, cryptographically random

STEP 2: Construct AAD
  aad = UTF-8(sender_address + receiver_address)
  // Concatenation of address strings, then UTF-8 encoded

STEP 3: AES-256-GCM encrypt
  (ciphertext, tag) = AES-256-GCM-ENCRYPT(
    key       = shared_secret,     // 32 bytes
    nonce     = iv,                // 12 bytes
    plaintext = plaintext,
    aad       = aad
  )
  // tag is 16 bytes

STEP 4: Assemble payload
  encrypted_payload = iv || ciphertext || tag
  // iv: bytes [0:12]
  // ciphertext: bytes [12 : len-16]
  // tag: bytes [len-16 : len]

STEP 5: Hash for header
  payload_hash = hex_encode(SHA-256(encrypted_payload))
  // 64 lowercase hex characters

OUTPUT: encrypted_payload (bytes), payload_hash (string)
```

### Decrypt

```
INPUT:  encrypted_payload (bytes), shared_secret (32 bytes),
        sender_address (string), receiver_address (string)

STEP 1: Extract components
  iv         = encrypted_payload[0 : 12]
  ciphertext = encrypted_payload[12 : len-16]
  tag        = encrypted_payload[len-16 : len]

STEP 2: Construct AAD
  aad = UTF-8(sender_address + receiver_address)

STEP 3: AES-256-GCM decrypt
  plaintext = AES-256-GCM-DECRYPT(
    key            = shared_secret,
    nonce          = iv,
    ciphertext_tag = ciphertext || tag,   // Some APIs take these separately
    aad            = aad
  )
  // If authentication fails: raise error, zero all buffers

OUTPUT: plaintext (bytes) or ERROR
```

---

## 7. The 8-Step Verification Gate

### Implementation Pseudocode

```
FUNCTION check_header(header, my_keypair, config, prev_sender_pubkey):
  report = new GateReport()

  // STEP 1: Structural Validation
  if header.version != 1: FAIL(report, "unsupported version")
  if len(header.sender_pubkey) != 66: FAIL(report, "bad pubkey length")
  if len(header.block_hash) != 64: FAIL(report, "bad block hash length")
  if len(header.payload_hash) != 64: FAIL(report, "bad payload hash length")
  if header.signature == "": FAIL(report, "missing signature")
  if NOT verify_address_pubkey_consistency(header.sender, header.sender_pubkey):
    FAIL(report, "pubkey does not derive to claimed address")
  report.structural = PASS

  // STEP 2: Receiver Match
  if header.receiver != my_keypair.address:
    FAIL(report, "not for this receiver")
  report.receiver_match = PASS

  // STEP 3: Signature Verification
  canonical = build_canonical_payload(header)
  message_hash = SHA-256(UTF-8(canonical))
  sig_bytes = Base64_Decode(header.signature)
  sender_pk = hex_decode(header.sender_pubkey)
  if NOT ECDSA_VERIFY(sender_pk, sig_bytes, message_hash):
    FAIL(report, "signature invalid")
  report.signature = PASS

  // STEP 4: Block Freshness
  age = current_chain_tip - header.block_height
  if age < 0 OR age > config.freshness_window:
    FAIL(report, "block age outside window")
  report.freshness = PASS

  // STEP 5: Proof of Satoshi (OPTIONAL)
  if config.pos_enabled:
    balance = blockchain.get_balance(header.sender)
    if balance < config.pos_min_satoshis:
      FAIL(report, "insufficient balance")
    report.pos = PASS
  else:
    report.pos = SKIP

  // STEP 6: Chain Proof
  if header.prev_addr != "":
    chain_hash = SHA-256(UTF-8(header.sender))
    proof_bytes = Base64_Decode(header.chain_proof)
    if NOT ECDSA_VERIFY(prev_sender_pubkey, proof_bytes, chain_hash):
      FAIL(report, "chain proof invalid")
    report.chain = PASS
  else:
    report.chain = PASS  // Genesis message, no proof needed

  return report  // gate_passed if all non-SKIP steps are PASS


FUNCTION accept_payload(header, encrypted_payload, my_keypair, report):
  if NOT report.gate_passed: return ERROR

  // STEP 7: Payload Hash
  computed = hex_encode(SHA-256(encrypted_payload))
  if computed != header.payload_hash:
    FAIL(report, "payload hash mismatch")
    return ERROR
  report.payload_hash = PASS

  // STEP 8: Decryption
  ecdh_pubkey = header.ephemeral_pubkey OR header.sender_pubkey
  ecdh_pk_bytes = hex_decode(ecdh_pubkey)
  shared_secret = ECDH(my_keypair.private_key, ecdh_pk_bytes)
  plaintext = AES_GCM_DECRYPT(encrypted_payload, shared_secret,
                               header.sender, header.receiver)
  if decryption_failed:
    FAIL(report, "decryption failed")
    return ERROR
  report.decryption = PASS

  return plaintext
```

---

## 8. Header Serialization

### Text Format (Email X-Headers / HTTP Headers)

```
BUTL-Version: 1
BUTL-Sender: 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
BUTL-SenderPubKey: 02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc
BUTL-Receiver: bc1q9h5yjqka3mz7f3z8v5lgcd0cugaay3m6ez3065
BUTL-ReceiverPubKey: 03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0
BUTL-BlockHeight: 890412
BUTL-BlockHash: 00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e
BUTL-PayloadHash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
BUTL-Signature: MEQCIGrD2w7h...
BUTL-PrevAddr:
BUTL-ChainProof:
BUTL-EncAlgo: AES-256-GCM

// Format: "Key: Value" (one space after colon)
// One field per line
// Optional fields included only if non-empty (except PrevAddr and ChainProof
// which are always included for the 12 required fields)
```

### JSON Format

```json
{
  "version": 1,
  "sender": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "sender_pubkey": "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc",
  "receiver": "bc1q9h5yjqka3mz7f3z8v5lgcd0cugaay3m6ez3065",
  "receiver_pubkey": "03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0",
  "block_height": 890412,
  "block_hash": "00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e",
  "payload_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "signature": "MEQCIGrD2w7h...",
  "prev_addr": "",
  "chain_proof": "",
  "enc_algo": "AES-256-GCM",
  "ephemeral_pubkey": "",
  "nonce": "",
  "timestamp": 0,
  "thread_id": "",
  "seq_num": 0,
  "payload_size": 0
}

// JSON field names: snake_case (hyphens removed, BUTL- prefix removed)
// Integers as numbers, strings as strings
// Empty strings for absent values (not null, not omitted)
```

---

## 9. Complete Send Flow

```
FUNCTION send_message(keychain, receiver_pubkey, receiver_address, body, config):

  // 1. Fresh keypair
  keypair = keychain.next_keypair()

  // 2. ECDH shared secret
  shared_secret = ECDH(keypair.private_key, receiver_pubkey)

  // 3. Encrypt
  encrypted_payload = AES_GCM_ENCRYPT(body, shared_secret,
                                       keypair.address, receiver_address)

  // 4. Hash
  payload_hash = hex_encode(SHA-256(encrypted_payload))

  // 5. Block reference
  (block_height, block_hash) = blockchain.get_current_block()

  // 6. Build header
  header = {
    version: 1,
    sender: keypair.address,
    sender_pubkey: keypair.public_key_hex,
    receiver: receiver_address,
    receiver_pubkey: hex_encode(receiver_pubkey),
    block_height: block_height,
    block_hash: block_hash,
    payload_hash: payload_hash,
    prev_addr: keychain.previous_address(),
    enc_algo: "AES-256-GCM",
    nonce: hex_encode(random_bytes(12)),
    timestamp: current_unix_time(),
    thread_id: keychain.thread_id(),
    seq_num: keychain.sequence_number(),
    payload_size: len(encrypted_payload),
  }

  // 7. Chain proof
  if keychain.has_previous():
    chain_proof = ECDSA_SIGN(
      keychain.previous_private_key(),
      SHA-256(UTF-8(keypair.address))
    )
    header.chain_proof = Base64_Encode(chain_proof)

  // 8. Sign canonical payload
  canonical = build_canonical_payload(header)
  message_hash = SHA-256(UTF-8(canonical))
  signature = ECDSA_SIGN(keypair.private_key, message_hash)
  header.signature = Base64_Encode(signature)

  return (header, encrypted_payload)
```

---

## 10. Complete Receive Flow

```
FUNCTION receive_message(header, encrypted_payload, my_keypair, config,
                         prev_sender_pubkey):

  // Phase 1: Header verification (no payload processed)
  report = check_header(header, my_keypair, config, prev_sender_pubkey)

  if NOT report.gate_passed:
    return (NULL, report)  // Payload never downloaded

  // Phase 2: Payload verification and decryption
  plaintext = accept_payload(header, encrypted_payload, my_keypair, report)

  if plaintext == ERROR:
    return (NULL, report)  // Payload zeroed and discarded

  return (plaintext, report)
```

---

## 11. Data Structure Definitions

### For Typed Languages (Rust, Go, TypeScript, Java, C#)

```
struct BUTLKeypair {
  private_key:     bytes[32]
  public_key:      bytes[33]       // compressed secp256k1
  public_key_hex:  string          // 66 hex chars
  address:         string          // Bitcoin P2PKH
}

struct BUTLKeychain {
  seed:            bytes[32]
  index:           uint32
  current:         BUTLKeypair?
  previous:        BUTLKeypair?
  history:         list<string>    // addresses
}

struct BUTLHeader {
  // Required
  version:         uint32          // currently 1
  sender:          string          // Bitcoin address
  sender_pubkey:   string          // 66 hex chars
  receiver:        string          // Bitcoin address
  receiver_pubkey: string          // 66 hex chars
  block_height:    uint64
  block_hash:      string          // 64 hex chars
  payload_hash:    string          // 64 hex chars
  signature:       string          // Base64(DER)
  prev_addr:       string          // address or ""
  chain_proof:     string          // Base64(DER) or ""
  enc_algo:        string          // "AES-256-GCM"

  // Optional
  ephemeral_pubkey: string         // 66 hex chars or ""
  nonce:           string          // hex or ""
  timestamp:       uint64          // Unix seconds
  thread_id:       string          // 64 hex chars or ""
  seq_num:         uint32          // 0-indexed
  payload_size:    uint64          // bytes
}

struct BUTLMessage {
  header:            BUTLHeader
  encrypted_payload: bytes
}

struct ProofOfSatoshiConfig {
  enabled:       bool              // default: false
  min_satoshis:  uint64            // default: 1, range: 1-100000000
}

enum CheckResult { PASS, FAIL, SKIP }

struct GateReport {
  structural:    CheckResult
  receiver_match: CheckResult
  signature:     CheckResult
  freshness:     CheckResult
  pos:           CheckResult
  chain:         CheckResult
  payload_hash:  CheckResult
  decryption:    CheckResult
  errors:        list<string>

  gate_passed():    bool  // all non-SKIP steps in 1-6 are PASS
  fully_verified(): bool  // gate_passed AND payload_hash==PASS AND decryption==PASS
}

trait BlockchainProvider {
  get_current_block() -> (uint64, string)
  verify_block(height: uint64, hash: string) -> bool
  get_chain_height() -> uint64
  check_balance(address: string, min_sats: uint64) -> bool
}
```

---

## 12. Test Vector Validation

After implementing, verify against these exact values:

```
Seed: SHA-256("butl-test-vector-sender-0")
  = 61a6eae659ceff3e6bd3570a98ca319660aa451e65f15325197c7b3a37b59552

Expected pubkey: 03aa6d590b9689180908401aaeff9e116526cbe90d0f414e851d09b4e4b8abacfc
Expected address: 14Zzmx3Mi773r4U9FjKHbtNJw1TVTZjyho

Receiver seed: SHA-256("butl-test-vector-receiver-0")
Expected address: 16J5UTAiCJYFfWYN35zPZEHP6zdfZS71iz

ECDH shared secret (both sides):
  b0ef30fcdb2e43e10df3d89f0ae75761a8e7ed9e660e7b8afbfef79ed9accac3

Body: "Hello from BUTL test vectors!"
Body hash: 20016ed6d12f5411a5dd5749cfb2db43412402db661d6ba1d56f62f660bfb3b1

Thread ID (SHA-256 of sender address):
  8aa0c2306d46c74e512b30b6967f9e7e5442e38ab78126fd935f0d10a221249b

Canonical payload hash (with block_height=890412, empty PrevAddr and Nonce):
  b9606411186a93804aa750f996a8b8cd4e8730a7912a3b1fbeccbb9de42c1864
```

If your implementation produces these exact values, it is compatible with every other correct BUTL implementation.

See [TEST_VECTORS.md](../spec/TEST_VECTORS.md) for the complete set of 11 test vectors.

---

## 13. Language-Specific Library Recommendations

| Language | secp256k1 | SHA-256 | AES-256-GCM | Base58 |
|----------|-----------|---------|-------------|--------|
| Python | `ecdsa` | `hashlib` (stdlib) | `pycryptodome` | Custom (see ref impl) |
| Rust | `secp256k1` (libsecp256k1 FFI) | `sha2` | `aes-gcm` | `bs58` |
| JavaScript | `secp256k1` or `elliptic` | `crypto` (Node) or `SubtleCrypto` (browser) | `crypto` (Node) or `SubtleCrypto` | `bs58` (npm) |
| Go | `btcd/btcec` | `crypto/sha256` | `crypto/cipher` | `btcutil/base58` |
| C | `libsecp256k1` | OpenSSL or mbedtls | OpenSSL or mbedtls | Custom |
| Java | BouncyCastle | `MessageDigest` | `javax.crypto` | Custom or BitcoinJ |
| Swift | `secp256k1.swift` | `CryptoKit` | `CryptoKit` | Custom |
| Kotlin | BouncyCastle | `MessageDigest` | `javax.crypto` | Custom or BitcoinJ |

For all languages: prefer libraries that wrap `libsecp256k1` (the C library used by Bitcoin Core) for constant-time secp256k1 operations.

---

*Every algorithm. Every byte format. Every data structure. An AI that follows this document will produce a correct, interoperable BUTL implementation in any language.*
