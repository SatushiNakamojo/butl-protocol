# BUTL Header Registry

## Canonical Registry of All BUTL Protocol Header Fields

This document is the authoritative reference for every `BUTL-*` header field name in the protocol. It defines the name, type, format, required/optional status, the protocol version that introduced it, and the verification step that uses it.

Implementations MUST NOT invent custom `BUTL-*` headers outside this registry. New headers require a protocol version increment (for required fields) or a spec revision (for optional fields) and must be added to this registry before use.

-----

## How to Read This Registry

|Column         |Meaning                                                                    |
|---------------|---------------------------------------------------------------------------|
|**Header Name**|The exact field name as it appears in the BUTL header. Case-sensitive.     |
|**Type**       |The data type of the value.                                                |
|**Format**     |The exact format specification.                                            |
|**Status**     |REQUIRED (must be present in every message) or OPTIONAL (may be present).  |
|**Since**      |The spec revision that introduced this field. All use protocol version `1`.|
|**Used In**    |Which verification step(s) use this field.                                 |
|**Description**|What the field is for.                                                     |

-----

## Required Fields

These fields MUST be present in every BUTL message. A message missing any required field MUST be rejected at Step 1 (Structural Validation).

### BUTL-Version

|               |                                                                                                                                                                                                                                                                                                                                            |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |Integer                                                                                                                                                                                                                                                                                                                                     |
|**Format**     |Currently `1`                                                                                                                                                                                                                                                                                                                               |
|**Status**     |REQUIRED                                                                                                                                                                                                                                                                                                                                    |
|**Since**      |v1.0                                                                                                                                                                                                                                                                                                                                        |
|**Used In**    |Step 1 (Structural Validation)                                                                                                                                                                                                                                                                                                              |
|**Description**|Protocol version number. Receivers MUST reject messages with unsupported versions. All v1.x spec revisions (v1.0, v1.1, v1.2) use protocol version `1`. A new protocol version (e.g., `2`) is required only when the canonical signing payload format changes, required fields are added or removed, or the verification step order changes.|

### BUTL-Sender

|               |                                                                                                                                                                                                  |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                            |
|**Format**     |Bitcoin address (Base58Check P2PKH starting with `1`, or Bech32 SegWit starting with `bc1`)                                                                                                       |
|**Status**     |REQUIRED                                                                                                                                                                                          |
|**Since**      |v1.0                                                                                                                                                                                              |
|**Used In**    |Step 1 (format validation), Step 3 (consistency check: must match BUTL-SenderPubKey), Step 5 (Proof of Satoshi balance query), Step 8 (AAD binding for AES-GCM), address chaining                 |
|**Description**|The sender’s Bitcoin address for this message. In address chaining, this is a fresh address for each message. The receiver verifies that this address is correctly derived from BUTL-SenderPubKey.|

### BUTL-SenderPubKey

|               |                                                                                                                                                                                                                            |
|---------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                      |
|**Format**     |66 hexadecimal characters (33-byte compressed secp256k1 public key: prefix `02` or `03` + 32-byte x-coordinate)                                                                                                             |
|**Status**     |REQUIRED                                                                                                                                                                                                                    |
|**Since**      |v1.0                                                                                                                                                                                                                        |
|**Used In**    |Step 1 (format validation, decompression check), Step 3 (ECDSA signature verification), Step 8 (ECDH shared secret computation, unless BUTL-EphemeralPubKey is present)                                                     |
|**Description**|The sender’s compressed secp256k1 public key. Used for signature verification and, when no ephemeral key is present, for ECDH key agreement. The receiver MUST verify that this public key correctly derives to BUTL-Sender.|

### BUTL-Receiver

|               |                                                                                                                                                                                         |
|---------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                   |
|**Format**     |Bitcoin address (Base58Check or Bech32)                                                                                                                                                  |
|**Status**     |REQUIRED                                                                                                                                                                                 |
|**Since**      |v1.1                                                                                                                                                                                     |
|**Used In**    |Step 2 (Receiver Match: compared to receiver’s own address), Step 8 (AAD binding for AES-GCM)                                                                                            |
|**Description**|The intended receiver’s Bitcoin address. If this does not match the receiver’s own address, the message is not for this device and MUST be rejected at Step 2 without further processing.|

### BUTL-ReceiverPubKey

|               |                                                                                                                                                                                                                                                                                                                          |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                                                                    |
|**Format**     |66 hexadecimal characters (33-byte compressed secp256k1 public key)                                                                                                                                                                                                                                                       |
|**Status**     |REQUIRED                                                                                                                                                                                                                                                                                                                  |
|**Since**      |v1.1                                                                                                                                                                                                                                                                                                                      |
|**Used In**    |Sender-side only (used by the sender for ECDH key agreement to compute the shared secret)                                                                                                                                                                                                                                 |
|**Description**|The receiver’s compressed secp256k1 public key. The sender uses this to compute the ECDH shared secret for payload encryption. The receiver already has their own public key and does not need to read this field for verification, but it is included in the canonical signing payload and thus covered by the signature.|

### BUTL-BlockHeight

|               |                                                                                                                                                                                                             |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |Integer                                                                                                                                                                                                      |
|**Format**     |Non-negative integer                                                                                                                                                                                         |
|**Status**     |REQUIRED                                                                                                                                                                                                     |
|**Since**      |v1.0                                                                                                                                                                                                         |
|**Used In**    |Step 4 (Freshness: `0 ≤ (current_tip - BlockHeight) ≤ freshness_window`)                                                                                                                                     |
|**Description**|The Bitcoin block height referenced for freshness. The sender sets this to a recent block height at the time of message creation. The receiver compares it to the current chain tip to determine message age.|

### BUTL-BlockHash

|               |                                                                                                                                                                                                                                               |
|---------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                         |
|**Format**     |64 hexadecimal characters (32-byte SHA-256d hash of the block header)                                                                                                                                                                          |
|**Status**     |REQUIRED                                                                                                                                                                                                                                       |
|**Since**      |v1.0                                                                                                                                                                                                                                           |
|**Used In**    |Step 4 (Freshness: receiver verifies hash matches the block at the claimed height)                                                                                                                                                             |
|**Description**|The hash of the Bitcoin block at BUTL-BlockHeight. This allows the receiver to verify that the claimed block height corresponds to a real block on the canonical chain, preventing an attacker from using an arbitrary height with a fake hash.|

### BUTL-PayloadHash

|               |                                                                                                                                                                                                                                                                                                        |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                                                  |
|**Format**     |64 hexadecimal characters (32-byte SHA-256 hash of the encrypted payload)                                                                                                                                                                                                                               |
|**Status**     |REQUIRED                                                                                                                                                                                                                                                                                                |
|**Since**      |v1.0                                                                                                                                                                                                                                                                                                    |
|**Used In**    |Step 7 (Payload Hash: receiver computes SHA-256 of the received encrypted payload and compares)                                                                                                                                                                                                         |
|**Description**|The SHA-256 hash of the encrypted payload (the entire blob: IV + ciphertext + GCM tag). The receiver uses this to verify that the payload was not tampered with in transit before attempting decryption. This field is part of the canonical signing payload and thus covered by the sender’s signature.|

### BUTL-Signature

|               |                                                                                                                                                                                                                                                                                               |
|---------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                                         |
|**Format**     |Base64-encoded DER-format ECDSA signature                                                                                                                                                                                                                                                      |
|**Status**     |REQUIRED                                                                                                                                                                                                                                                                                       |
|**Since**      |v1.0                                                                                                                                                                                                                                                                                           |
|**Used In**    |Step 3 (Signature Verification)                                                                                                                                                                                                                                                                |
|**Description**|The sender’s ECDSA signature over the SHA-256 hash of the canonical signing payload (see [Specification §7](BUTL_Protocol_Specification_v1.2.md)). This proves the sender controls the private key corresponding to BUTL-SenderPubKey and that no header field has been modified since signing.|

### BUTL-PrevAddr

|               |                                                                                                                                                                                                       |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                 |
|**Format**     |Bitcoin address (Base58Check or Bech32) or empty string                                                                                                                                                |
|**Status**     |REQUIRED                                                                                                                                                                                               |
|**Since**      |v1.0                                                                                                                                                                                                   |
|**Used In**    |Step 6 (Chain Proof: identifies which previous public key to verify against)                                                                                                                           |
|**Description**|The sender’s Bitcoin address from the previous message in this conversation thread. Empty string for genesis messages (first message in a chain). When non-empty, BUTL-ChainProof MUST also be present.|

### BUTL-ChainProof

|               |                                                                                                                                                                                                                                                                                         |
|---------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                                   |
|**Format**     |Base64-encoded DER-format ECDSA signature, or empty string                                                                                                                                                                                                                               |
|**Status**     |REQUIRED                                                                                                                                                                                                                                                                                 |
|**Since**      |v1.0                                                                                                                                                                                                                                                                                     |
|**Used In**    |Step 6 (Chain Proof Verification)                                                                                                                                                                                                                                                        |
|**Description**|An ECDSA signature where the previous message’s private key signs `SHA-256(UTF-8(BUTL-Sender))`. This proves that the entity controlling the previous address also controls the new address, providing verifiable identity continuity across fresh addresses. Empty for genesis messages.|

### BUTL-EncAlgo

|               |                                                                                                                                                                                                                                |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                          |
|**Format**     |Algorithm identifier string                                                                                                                                                                                                     |
|**Status**     |REQUIRED                                                                                                                                                                                                                        |
|**Since**      |v1.1                                                                                                                                                                                                                            |
|**Used In**    |Step 8 (Decryption: determines which algorithm to use)                                                                                                                                                                          |
|**Description**|The encryption algorithm used for the payload. Default and currently only supported value: `AES-256-GCM`. Future versions may add additional algorithms. Receivers MUST reject messages with unrecognized algorithm identifiers.|

-----

## Optional Fields

These fields MAY be present. Receivers MUST ignore unrecognized optional fields (forward compatibility). The absence of an optional field MUST NOT cause verification failure.

### BUTL-EphemeralPubKey

|               |                                                                                                                                                                                                                                           |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                     |
|**Format**     |66 hexadecimal characters (33-byte compressed secp256k1 public key)                                                                                                                                                                        |
|**Status**     |OPTIONAL                                                                                                                                                                                                                                   |
|**Since**      |v1.1                                                                                                                                                                                                                                       |
|**Used In**    |Step 8 (ECDH: when present, receiver uses this key instead of BUTL-SenderPubKey for shared secret computation)                                                                                                                             |
|**Description**|An ephemeral public key generated by the sender specifically for this message’s ECDH key agreement. Separates the signing key from the encryption key, providing enhanced forward secrecy. RECOMMENDED as a best practice for all messages.|

### BUTL-Nonce

|               |                                                                                                                                                                                                                                           |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                     |
|**Format**     |Hexadecimal-encoded random bytes (recommended: 24 hex chars / 12 bytes)                                                                                                                                                                    |
|**Status**     |OPTIONAL                                                                                                                                                                                                                                   |
|**Since**      |v1.0                                                                                                                                                                                                                                       |
|**Used In**    |Part of the canonical signing payload (covered by signature). Additional anti-replay protection beyond block height freshness.                                                                                                             |
|**Description**|A random nonce that ensures two messages sent within the same block have different canonical signing payloads and therefore different signatures. Combined with block height freshness, this provides comprehensive anti-replay protection.|

### BUTL-Timestamp

|               |                                                                                                                                                                                                                                 |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |Integer                                                                                                                                                                                                                          |
|**Format**     |Unix epoch seconds (non-negative)                                                                                                                                                                                                |
|**Status**     |OPTIONAL                                                                                                                                                                                                                         |
|**Since**      |v1.0                                                                                                                                                                                                                             |
|**Used In**    |Informational only. Not used in any verification step.                                                                                                                                                                           |
|**Description**|The Unix timestamp of message creation. Informational — useful for logging, display, and debugging. Not relied upon for security because system clocks can be manipulated. Block height is the authoritative freshness mechanism.|

### BUTL-ThreadID

|               |                                                                                                                                                                                                                                                                        |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                  |
|**Format**     |64 hexadecimal characters (SHA-256 hash of the first sender address in the chain)                                                                                                                                                                                       |
|**Status**     |OPTIONAL                                                                                                                                                                                                                                                                |
|**Since**      |v1.0                                                                                                                                                                                                                                                                    |
|**Used In**    |Application-layer conversation threading. Not used in verification.                                                                                                                                                                                                     |
|**Description**|Identifies the conversation thread. Computed as `SHA-256_HEX(UTF-8(Address_0))` where `Address_0` is the sender’s address in the genesis message. Constant across all messages in the chain. Useful for organizing messages into conversations at the application layer.|

### BUTL-SeqNum

|               |                                                                                                                                                                                                                      |
|---------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |Integer                                                                                                                                                                                                               |
|**Format**     |Non-negative integer, 0-indexed                                                                                                                                                                                       |
|**Status**     |OPTIONAL                                                                                                                                                                                                              |
|**Since**      |v1.0                                                                                                                                                                                                                  |
|**Used In**    |Application-layer message ordering. Not used in verification.                                                                                                                                                         |
|**Description**|The sequence number of this message within its thread. 0 for the genesis message, incrementing by 1 for each subsequent message. Useful for detecting missing messages and maintaining order at the application layer.|

### BUTL-PayloadSize

|               |                                                                                                                                                                                                                                                                                                               |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |Integer                                                                                                                                                                                                                                                                                                        |
|**Format**     |Non-negative integer (bytes)                                                                                                                                                                                                                                                                                   |
|**Status**     |OPTIONAL                                                                                                                                                                                                                                                                                                       |
|**Since**      |v1.1                                                                                                                                                                                                                                                                                                           |
|**Used In**    |Pre-download validation. The receiver MAY reject messages with payloads exceeding a configured maximum size before downloading the payload.                                                                                                                                                                    |
|**Description**|The size of the encrypted payload in bytes. Allows the receiver to enforce size limits and allocate resources before Phase 2 begins. If present and the receiver has a maximum payload size policy, the receiver MAY reject messages where BUTL-PayloadSize exceeds the limit, without downloading the payload.|

### BUTL-BalanceProof

|               |                                                                                                                                                                                                                                                                                                                              |
|---------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|**Type**       |String                                                                                                                                                                                                                                                                                                                        |
|**Format**     |Implementation-defined (SPV proof, API reference, signed attestation, etc.)                                                                                                                                                                                                                                                   |
|**Status**     |OPTIONAL                                                                                                                                                                                                                                                                                                                      |
|**Since**      |v1.0                                                                                                                                                                                                                                                                                                                          |
|**Used In**    |Step 5 (Proof of Satoshi: may be used for offline balance verification instead of a live blockchain query)                                                                                                                                                                                                                    |
|**Description**|A proof that the sender’s address holds the required balance, for scenarios where the receiver cannot perform a live blockchain query. The format is implementation-defined. One approach: a Merkle inclusion proof showing the sender’s UTXO exists in a specific block, verified against a block header the receiver trusts.|

-----

## Reserved Fields

These field names are reserved for future versions of the protocol. They MUST NOT be used by implementations until formally defined in a future spec revision.

|Header Name     |Planned Purpose                                                 |Target Version|
|----------------|----------------------------------------------------------------|--------------|
|BUTL-Revocation |Signal that a key or ThreadID has been revoked due to compromise|v1.3 / v2.0   |
|BUTL-MultiSig   |Support for messages requiring multiple signers                 |v2.0          |
|BUTL-PostQuantum|Algorithm negotiation for post-quantum hybrid mode              |v2.0          |
|BUTL-Stream     |Streaming mode identifier for video/voice/live data             |v3.0+         |
|BUTL-GroupID    |Group conversation identifier for multi-party messaging         |v2.0          |

-----

## Field Usage by Verification Step

This table shows which header fields are read at each step of the verification sequence.

|Step               |Fields Used                                                                                                                                                                                       |
|-------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|1. Structure       |All required fields (presence and format check)                                                                                                                                                   |
|2. Receiver Match  |BUTL-Receiver                                                                                                                                                                                     |
|3. Signature       |BUTL-SenderPubKey, BUTL-Signature, and all fields in the canonical signing payload (Version, Sender, SenderPubKey, Receiver, ReceiverPubKey, BlockHeight, BlockHash, PayloadHash, PrevAddr, Nonce)|
|4. Freshness       |BUTL-BlockHeight, BUTL-BlockHash                                                                                                                                                                  |
|5. Proof of Satoshi|BUTL-Sender (for balance query), BUTL-BalanceProof (if present, for offline verification)                                                                                                         |
|6. Chain Proof     |BUTL-PrevAddr, BUTL-ChainProof, BUTL-Sender                                                                                                                                                       |
|7. Payload Hash    |BUTL-PayloadHash                                                                                                                                                                                  |
|8. Decryption      |BUTL-EphemeralPubKey (if present) or BUTL-SenderPubKey, BUTL-Sender, BUTL-Receiver (for AAD), BUTL-EncAlgo                                                                                        |

-----

## Canonical Signing Payload Fields

The following fields are included in the canonical signing payload (in this exact order) and are therefore covered by BUTL-Signature:

1. BUTL-Version
1. BUTL-Sender
1. BUTL-SenderPubKey
1. BUTL-Receiver
1. BUTL-ReceiverPubKey
1. BUTL-BlockHeight
1. BUTL-BlockHash
1. BUTL-PayloadHash
1. BUTL-PrevAddr
1. BUTL-Nonce

Any modification to any of these fields after signing will cause signature verification to fail at Step 3.

-----

## Serialization Rules

### Text-Based Protocols (Email, HTTP)

```
BUTL-Version: 1
BUTL-Sender: bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh
BUTL-SenderPubKey: 02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc
```

- Format: `Key: Value` (one space after the colon)
- One field per line
- UTF-8 encoding
- Lines terminated by CRLF (`\r\n`) for email (per RFC 5322), LF (`\n`) for HTTP headers

### JSON Serialization

```json
{
  "version": 1,
  "sender": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
  "sender_pubkey": "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc",
  "receiver": "bc1q9h5yjqka3mz7f3z8v5lgcd0cugaay3m6ez3065",
  "receiver_pubkey": "03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0",
  "block_height": 890412,
  "block_hash": "00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e",
  "payload_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "signature": "MEQCIGrD2w7h...",
  "prev_addr": "",
  "chain_proof": "",
  "enc_algo": "AES-256-GCM"
}
```

JSON field names use `snake_case` (replacing hyphens and removing the `BUTL-` prefix).

-----

## Rules Summary

1. All BUTL headers MUST use the `BUTL-` prefix.
1. Header names are case-sensitive: `BUTL-Sender` is valid, `butl-sender` is not.
1. Implementations MUST ignore unrecognized `BUTL-*` headers (forward compatibility).
1. Implementations MUST NOT invent new `BUTL-*` headers outside this registry.
1. New required headers require a protocol version increment.
1. New optional headers require a spec revision and an update to this registry.
1. Receivers MUST verify BUTL-SenderPubKey derives to BUTL-Sender (consistency check).
1. All hex values are lowercase.
1. Empty strings are used for absent optional values in the canonical signing payload, not omitted fields.

-----

*This registry is the single source of truth for BUTL header fields. If it’s not listed here, it’s not a valid BUTL header.*