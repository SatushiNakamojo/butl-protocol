# BUTL — Bitcoin Universal Trust Layer

## Protocol Specification v1.2

*A cryptographic identity, encryption, and trust layer for the open internet, anchored to the Bitcoin blockchain.*

**Status:** Published
**Date:** March 2026
**License:** MIT OR Apache 2.0 (dual licensed)
**Patent:** PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md (applies to all users)

-----

## Table of Contents

1. Abstract
1. Design Principles
1. Terminology
1. Protocol Overview
1. Identity
1. Header Specification
1. Canonical Signing Payload
1. End-to-End Encryption
1. Address Chaining
1. Proof of Satoshi
1. Verification Sequence
1. Key Discovery
1. Transport and Application Bindings
1. Security Considerations
1. Cryptographic Primitives
1. Implementation Notes

-----

## 1. Abstract

The Bitcoin Universal Trust Layer (BUTL) is a lightweight cryptographic protocol that provides identity verification, end-to-end encryption, message integrity, freshness assurance, forward privacy, and optional sybil resistance for any internet communication. It uses Bitcoin’s existing infrastructure — SHA-256 hashing, secp256k1 ECDSA/ECDH, blockchain block heights, and optionally UTXO balance checks — as a universal trust anchor.

BUTL enforces a strict verify-before-download security model: no payload data is written to the receiving device until all header verification checks have passed.

The protocol requires no changes to Bitcoin, no new blockchain, and no tokens. It composes existing, battle-tested primitives into a universal trust layer that any internet application can adopt.

**v1.2 changes:** The balance check (Proof of Satoshi) is explicitly OPTIONAL. Receivers configure whether to enforce it and at what threshold. When disabled, BUTL operates as a pure-math cryptographic trust layer with zero external dependencies beyond block height. Dual licensed under MIT OR Apache 2.0.

-----

## 2. Design Principles

**Simplicity.** Easy to implement in any language on any platform. The minimum implementation requires only SHA-256, secp256k1, AES-256-GCM, and Base58Check.

**Self-Sovereign Identity.** Identity is a Bitcoin address derived from a keypair the holder generates. No registration, no accounts, no intermediaries. No one can create, revoke, or control the identity except the key holder.

**End-to-End Encryption.** The payload is encrypted to the receiver’s public key via ECDH. No intermediary can read it. Not the transport. Not the relay. Not the ISP.

**Verify-Before-Download.** No payload data touches the receiving device until all header checks pass. This is a hard security boundary, not a suggestion.

**Forward Privacy.** A new address is used for every message, chained to the previous via a cryptographic proof. Third-party observers see unrelated addresses. Only the receiver can verify continuity.

**Pure Cryptography First.** The core protocol (steps 1-4, 6-8 of the verification sequence) is 100% local math. No external trust dependencies. Proof of Satoshi is the only step that requires an external query, and it is optional.

**Optional Sybil Resistance.** Proof of Satoshi provides economic identity gating for applications that need it. It is configurable by the receiver: toggle on or off, and set the minimum satoshi threshold with a slider.

**Bitcoin-Native.** Uses only primitives present in or compatible with Bitcoin: SHA-256, secp256k1, RIPEMD-160, Base58Check, Bech32, block heights, UTXO balances.

**Transport Agnostic.** BUTL is a trust layer, not a transport layer. It works over TCP, UDP, QUIC, HTTP, WebSocket, SMTP, MQTT, Bluetooth, NFC, or any mechanism that can move bytes.

-----

## 3. Terminology

The key words “MUST,” “MUST NOT,” “REQUIRED,” “SHALL,” “SHOULD,” “RECOMMENDED,” “MAY,” and “OPTIONAL” in this document are to be interpreted as described in RFC 2119.

Additional terms specific to this protocol are defined in [GLOSSARY.md](../GLOSSARY.md).

|Term                  |Definition                                                          |
|----------------------|--------------------------------------------------------------------|
|BUTL-ID               |A BUTL identity: private key + public key + Bitcoin address         |
|BUTL Header           |The cleartext metadata portion of a BUTL message                    |
|Encrypted Payload     |The AES-256-GCM encrypted message body                              |
|Gate                  |The verification boundary between header checks and payload delivery|
|Chain Proof           |An ECDSA signature linking a new address to the previous address    |
|ThreadID              |SHA-256 hash of the first sender address in a chain                 |
|Proof of Satoshi (PoS)|Optional balance check for sybil resistance                         |
|Freshness Window      |Maximum block age accepted (default: 144 blocks)                    |
|Phase 1               |Header delivery and verification (no payload)                       |
|Phase 2               |Payload delivery after gate passes                                  |

-----

## 4. Protocol Overview

### 4.1 Architecture

```
┌─────────────────────────────────────────────────────┐
│               APPLICATION LAYER                     │
│   Email · Chat · API · Login · VoIP · IoT · Files   │
├─────────────────────────────────────────────────────┤
│               BUTL PROTOCOL (this spec)             │
│   Identity · Encryption · Integrity · Freshness     │
│   Verify-Before-Download · Address Chaining         │
│   [Optional: Proof of Satoshi]                      │
├─────────────────────────────────────────────────────┤
│               BITCOIN NETWORK                       │
│   SHA-256 · secp256k1 · ECDH · Blockchain           │
├─────────────────────────────────────────────────────┤
│               TRANSPORT (TCP/UDP/QUIC/HTTP/...)     │
└─────────────────────────────────────────────────────┘
```

### 4.2 Message Structure

Every BUTL message consists of two parts:

1. **BUTL Header** — cleartext metadata containing all information needed for the verification gate
1. **Encrypted Payload** — AES-256-GCM encrypted message body, transmitted only after the gate passes

### 4.3 Message Flow

```
SENDER                                            RECEIVER
  │                                                  │
  │  1. Generate fresh keypair                        │
  │  2. ECDH shared secret with receiver's pubkey     │
  │  3. AES-256-GCM encrypt the body                  │
  │  4. Build canonical signing payload                │
  │  5. ECDSA sign the payload                        │
  │  6. Create chain proof (if chained)               │
  │                                                  │
  │  ══════ PHASE 1: HEADER ONLY ══════              │
  │──── BUTL Header (cleartext) ────────────────────>│
  │                                                  │
  │              [Receiver runs BUTL Gate]            │
  │              [Steps 1-4, optional 5, 6]          │
  │              [ANY FAIL → drop connection]         │
  │              [ALL PASS → gate opens]              │
  │                                                  │
  │  ══════ PHASE 2: PAYLOAD (conditional) ══════    │
  │<──────────── BUTL-READY ─────────────────────────│
  │──── Encrypted Payload ──────────────────────────>│
  │              [Step 7: Payload hash]              │
  │              [Step 8: Decrypt]                   │
  │              [PASS → deliver to application]     │
  │              [FAIL → zero and discard]           │
```

-----

## 5. Identity

### 5.1 Definition

A BUTL identity (BUTL-ID) consists of three mathematically linked components:

1. **Private Key** — A 256-bit integer on the secp256k1 curve. The root of the identity. Never shared. Never transmitted. Never stored in plaintext.
1. **Public Key** — A point on secp256k1 derived from the private key via scalar multiplication (`PubKey = PrivKey × G`). Transmitted in compressed form (33 bytes: prefix byte `02` or `03` + 32-byte x-coordinate). Used for signature verification, ECDH encryption, and chain proof validation.
1. **Bitcoin Address** — A human-readable encoding of the public key hash. Derived through `Base58Check(0x00 || RIPEMD-160(SHA-256(CompressedPubKey)))` for P2PKH addresses (prefix “1”) or Bech32 encoding for SegWit addresses (prefix “bc1”).

### 5.2 Self-Sovereign Property

The identity is generated by the holder without registering with any authority. It can be independently verified by anyone with the public key. It cannot be revoked, suspended, or modified by any third party. Loss of the private key results in permanent, irrecoverable loss of the identity.

### 5.3 Why Both Address and Public Key

The address and public key serve different purposes and both MUST be present in the header:

- The **public key** is required for all cryptographic operations: ECDSA verification, ECDH key agreement, and chain proof verification.
- The **address** is required for receiver matching, Proof of Satoshi balance queries, AAD binding in encryption, human-readable identification, and error detection via its built-in checksum.

The address is derived from the public key through a one-way hash. The public key cannot be recovered from the address. Both must be transmitted.

### 5.4 Consistency Check

Receivers MUST verify that `BUTL-SenderPubKey` correctly derives to `BUTL-Sender`. If the address does not match the public key, the message MUST be rejected.

-----

## 6. Header Specification

### 6.1 Required Fields

|Field              |Type   |Format                  |Description                                                        |
|-------------------|-------|------------------------|-------------------------------------------------------------------|
|BUTL-Version       |Integer|`1`                     |Protocol version number.                                           |
|BUTL-Sender        |String |Bitcoin address         |Sender’s Bitcoin address for this message.                         |
|BUTL-SenderPubKey  |String |66 hex chars            |Sender’s compressed secp256k1 public key.                          |
|BUTL-Receiver      |String |Bitcoin address         |Intended receiver’s Bitcoin address.                               |
|BUTL-ReceiverPubKey|String |66 hex chars            |Receiver’s compressed secp256k1 public key.                        |
|BUTL-BlockHeight   |Integer|Non-negative            |Bitcoin block height referenced for freshness.                     |
|BUTL-BlockHash     |String |64 hex chars            |SHA-256d hash of the block at BUTL-BlockHeight.                    |
|BUTL-PayloadHash   |String |64 hex chars            |SHA-256 hash of the encrypted payload.                             |
|BUTL-Signature     |String |Base64 (DER)            |ECDSA signature over the canonical signing payload.                |
|BUTL-PrevAddr      |String |Bitcoin address or empty|Sender’s address from the previous message. Empty for genesis.     |
|BUTL-ChainProof    |String |Base64 (DER) or empty   |ECDSA proof linking current address to previous. Empty for genesis.|
|BUTL-EncAlgo       |String |Algorithm ID            |Encryption algorithm. Default: `AES-256-GCM`.                      |

### 6.2 Optional Fields

|Field               |Type   |Format                 |Description                                                                                                   |
|--------------------|-------|-----------------------|--------------------------------------------------------------------------------------------------------------|
|BUTL-EphemeralPubKey|String |66 hex chars           |Ephemeral public key for ECDH (enhanced forward secrecy). When present, used instead of SenderPubKey for ECDH.|
|BUTL-Nonce          |String |Hex                    |Random nonce for additional anti-replay protection.                                                           |
|BUTL-Timestamp      |Integer|Unix epoch seconds     |Message creation timestamp. Informational; not used in verification.                                          |
|BUTL-ThreadID       |String |64 hex chars           |SHA-256 of the first sender address in the chain. Constant across all messages in a thread.                   |
|BUTL-SeqNum         |Integer|Non-negative, 0-indexed|Sequence number within the thread. 0 for genesis.                                                             |
|BUTL-PayloadSize    |Integer|Non-negative (bytes)   |Size of encrypted payload. Allows pre-download size validation.                                               |
|BUTL-BalanceProof   |String |Implementation-defined |UTXO proof for offline Proof of Satoshi verification.                                                         |

### 6.3 Field Rules

- All field names are case-sensitive: `BUTL-Sender` is valid, `butl-sender` is not.
- Receivers MUST ignore unrecognized `BUTL-*` fields (forward compatibility).
- Implementations MUST NOT invent new `BUTL-*` fields outside the [Header Registry](HEADER_REGISTRY.md).
- New required fields require a protocol version increment.
- In text-based protocols (email, HTTP), fields follow `Key: Value` format with a single space after the colon.

-----

## 7. Canonical Signing Payload

The canonical signing payload is a deterministic string constructed from header fields. It is hashed with SHA-256 and the hash is signed with ECDSA.

### 7.1 Format

```
BUTL-Version:{version}
BUTL-Sender:{sender_address}
BUTL-SenderPubKey:{sender_pubkey_hex}
BUTL-Receiver:{receiver_address}
BUTL-ReceiverPubKey:{receiver_pubkey_hex}
BUTL-BlockHeight:{block_height}
BUTL-BlockHash:{block_hash}
BUTL-PayloadHash:{payload_hash}
BUTL-PrevAddr:{prev_address_or_empty}
BUTL-Nonce:{nonce_or_empty}
```

### 7.2 Rules

- Fields are separated by a single newline character (`0x0A`).
- No spaces around colons.
- Empty strings are used for missing optional values (the field is present but the value is empty).
- The field order is fixed and MUST NOT be changed.

### 7.3 Signature

```
message_hash = SHA-256(UTF-8(canonical_payload))
signature = ECDSA_SIGN(message_hash, sender_private_key)
```

The signature is DER-encoded and Base64-encoded for transmission in the `BUTL-Signature` header field.

-----

## 8. End-to-End Encryption

### 8.1 Key Agreement (ECDH)

The sender and receiver derive an identical shared secret using Elliptic Curve Diffie-Hellman on secp256k1:

```
Sender:   shared_secret = SHA-256( (sender_private × receiver_public).x )
Receiver: shared_secret = SHA-256( (receiver_private × sender_public).x )
```

Both computations yield the same 32-byte value due to the commutative property of elliptic curve scalar multiplication. This shared secret serves as the AES-256-GCM encryption key.

### 8.2 Payload Encryption

1. Generate a random 12-byte IV.
1. Construct AAD: `UTF-8(sender_address || receiver_address)`.
1. Encrypt: `AES-256-GCM(key=shared_secret, iv=IV, plaintext=body, aad=AAD)`.
1. Construct encrypted payload: `IV (12 bytes) || ciphertext || GCM tag (16 bytes)`.
1. Compute: `BUTL-PayloadHash = SHA-256_HEX(encrypted_payload)`.

### 8.3 Payload Decryption

1. Extract: `IV = encrypted_payload[0:12]`, `ciphertext_and_tag = encrypted_payload[12:]`.
1. Construct AAD: `UTF-8(sender_address || receiver_address)`.
1. Decrypt: `AES-256-GCM-DECRYPT(key=shared_secret, iv=IV, ciphertext_and_tag, aad=AAD)`.
1. If GCM authentication fails, zero all buffers and reject.

### 8.4 AAD Binding

The Additional Authenticated Data binds the ciphertext to the sender and receiver addresses. If an attacker redirects the encrypted payload to a different receiver (changing the address), the GCM authentication check fails and decryption is rejected. This prevents payload redirection attacks.

### 8.5 Ephemeral Keys

The sender MAY generate an ephemeral keypair for ECDH and include the ephemeral public key in `BUTL-EphemeralPubKey`. When present, the receiver MUST use the ephemeral key (not `BUTL-SenderPubKey`) for ECDH.

This separates the signing key from the encryption key, providing enhanced forward secrecy. Compromise of the signing key does not retroactively compromise message confidentiality.

-----

## 9. Address Chaining

### 9.1 Mechanism

Each message uses a fresh Bitcoin address derived from the next key in a deterministic sequence (BIP-32 hierarchical derivation RECOMMENDED for production). To prove continuity, the sender signs the new address with the previous message’s private key:

```
chain_proof = ECDSA_SIGN(SHA-256(UTF-8(new_address)), previous_private_key)
```

### 9.2 Chain Structure

```
Message 0:  Addr_0  (genesis, PrevAddr = "", ChainProof = "")
Message 1:  Addr_1  (PrevAddr = Addr_0, ChainProof = Sign(Addr_1, key_0))
Message 2:  Addr_2  (PrevAddr = Addr_1, ChainProof = Sign(Addr_2, key_1))
    ...
Message N:  Addr_N  (PrevAddr = Addr_{N-1}, ChainProof = Sign(Addr_N, key_{N-1}))
```

### 9.3 Thread Identity

```
ThreadID = SHA-256_HEX(UTF-8(Addr_0))
```

The ThreadID is constant across all messages in the chain. It uniquely identifies the conversation thread.

### 9.4 Properties

- **Forward secrecy.** Compromising the current key does not reveal past keys.
- **Unlinkability.** Third-party observers see unrelated addresses. Without the chain proofs, different messages cannot be linked to the same sender.
- **Verifiable continuity.** The receiver confirms that consecutive messages come from the same entity by verifying the chain proof against the previous sender’s public key.
- **Automatic key rotation.** Every message is a key rotation event.

### 9.5 Key Deletion

After creating the chain proof, the previous private key SHOULD be securely deleted (memory zeroed). Only the current private key needs to be retained.

-----

## 10. Proof of Satoshi

### 10.1 Overview

Proof of Satoshi (PoS) is an OPTIONAL verification step that checks whether the sender’s Bitcoin address holds a minimum balance. It provides sybil resistance by attaching an economic cost to identity creation.

PoS is configured entirely by the receiver. The protocol defines the mechanism. The receiver decides whether to use it.

### 10.2 Receiver Configuration

|Setting           |Type   |Default|Description                                              |
|------------------|-------|-------|---------------------------------------------------------|
|`pos_enabled`     |Boolean|`false`|Whether to enforce balance checking.                     |
|`pos_min_satoshis`|Integer|`1`|Minimum balance required. Range: 1 — 2,100,000,000,000,000 (21,000,000 BTC, total supply). Implementations SHOULD use a logarithmic scale for UI sliders.|

### 10.3 When PoS Is Enabled

The receiver queries the Bitcoin blockchain (full node, Electrum server, or API) to verify that `BUTL-Sender` holds at least `pos_min_satoshis`. If the check fails, the gate closes and the message is rejected.

### 10.4 When PoS Is Disabled

Step 5 of the verification sequence is skipped entirely. The gate runs steps 1-4 and 6 only. The protocol provides identity, integrity, encryption, freshness, and chain verification — but NOT sybil resistance.

### 10.5 Recommended Thresholds

|Use Case               |Suggested Threshold|Rationale                           |
|-----------------------|-------------------|------------------------------------|
|Casual messaging       |OFF                |Signature alone proves key ownership|
|IoT device registration|100 sat            |Low barrier, minimal cost           |
|Public API             |1,000 sat          |Deters casual abuse                 |
|Social media posting   |10,000 sat         |Makes bot farms expensive           |
|Enterprise API         |100,000 sat        |Ensures funded, serious entities    |
|Financial systems      |1,000,000 sat      |High economic stake required        |

### 10.6 Security Note

When PoS is disabled, an attacker can create unlimited BUTL identities at zero economic cost (generating keypairs is free). Each identity will have valid signatures, valid encryption, and valid chain proofs. The only missing property is economic stake. Applications that require sybil resistance MUST enable PoS or implement alternative mechanisms at the application layer.

-----

## 11. Verification Sequence

### 11.1 Overview

|#|Check                |Required?   |What It Proves                           |On Fail|
|-|---------------------|------------|-----------------------------------------|-------|
|1|Structural Validation|REQUIRED    |Header well-formed, fields present       |REJECT |
|2|Receiver Match       |REQUIRED    |Message is addressed to this device      |REJECT |
|3|Signature (ECDSA)    |REQUIRED    |Sender controls the claimed key          |REJECT |
|4|Block Freshness      |REQUIRED    |Message is recent (anti-replay)          |REJECT |
|5|Proof of Satoshi     |**OPTIONAL**|Economic stake (sybil resistance)        |REJECT*|
|6|Chain Proof          |REQUIRED**  |Same sender as previous message          |REJECT |
|—|**— GATE —**         |            |**— PAYLOAD DOWNLOAD BEGINS —**          |       |
|7|Payload Hash         |REQUIRED    |Encrypted payload not tampered in transit|DISCARD|
|8|Decryption (GCM)     |REQUIRED    |Only intended receiver can read payload  |DISCARD|

*Step 5 only runs if `pos_enabled = true` in receiver configuration.
**Step 6 only applies to threaded messages (`BUTL-PrevAddr` is non-empty).

### 11.2 Step Details

**Step 1: Structural Validation.** Verify that all required fields are present, `BUTL-Version` is a supported value, `BUTL-BlockHash` is 64 hex characters, `BUTL-PayloadHash` is 64 hex characters, `BUTL-SenderPubKey` is a valid compressed secp256k1 point, and `BUTL-SenderPubKey` correctly derives to `BUTL-Sender` (consistency check).

**Step 2: Receiver Match.** Verify that `BUTL-Receiver` equals the receiver’s own Bitcoin address. If not, this message is not for this device — reject without further processing.

**Step 3: Signature Verification.** Reconstruct the canonical signing payload from header fields (Section 7). Compute `message_hash = SHA-256(UTF-8(canonical_payload))`. Verify the ECDSA signature in `BUTL-Signature` against `BUTL-SenderPubKey` and `message_hash`. If invalid, the sender does not control the claimed key.

**Step 4: Block Freshness.** Verify that `BUTL-BlockHash` matches the Bitcoin block at height `BUTL-BlockHeight`. Compute `age = current_chain_tip - BUTL-BlockHeight`. Verify `0 ≤ age ≤ freshness_window` (default: 144 blocks, approximately 24 hours). If the age exceeds the window, the message may be a replay.

**Step 5: Proof of Satoshi (OPTIONAL).** If `pos_enabled = true`: query the Bitcoin blockchain for the balance of `BUTL-Sender`. Verify `balance ≥ pos_min_satoshis`. If `pos_enabled = false`: skip this step entirely.

**Step 6: Chain Proof.** If `BUTL-PrevAddr` is non-empty: compute `chain_hash = SHA-256(UTF-8(BUTL-Sender))`. Verify the ECDSA signature in `BUTL-ChainProof` against the previous sender’s public key (from the previous verified message) and `chain_hash`. If `BUTL-PrevAddr` is empty (genesis message): skip this step.

**Step 7: Payload Hash.** Compute `SHA-256_HEX(encrypted_payload)` and compare to `BUTL-PayloadHash`. If they differ, the payload was tampered with in transit. Zero and discard.

**Step 8: Decryption.** Determine the ECDH public key: use `BUTL-EphemeralPubKey` if present, otherwise `BUTL-SenderPubKey`. Compute the ECDH shared secret. Decrypt the payload with AES-256-GCM. If GCM authentication fails, the shared secret is wrong (message was not encrypted to this receiver) or the payload is corrupted. Zero and discard.

### 11.3 Pure Cryptography Mode (PoS Disabled)

When Proof of Satoshi is disabled, the gate runs: 1 → 2 → 3 → 4 → 6 → **GATE** → 7 → 8.

Every check in this sequence is pure mathematics performed locally:

|Step|Operation                                     |External Query?|
|----|----------------------------------------------|---------------|
|1   |String validation, pubkey decompression       |No             |
|2   |String comparison                             |No             |
|3   |SHA-256 + ECDSA verification                  |No             |
|4   |Integer comparison against cached block height|No             |
|6   |SHA-256 + ECDSA verification                  |No             |
|7   |SHA-256 comparison                            |No             |
|8   |ECDH + AES-256-GCM decryption                 |No             |

No network calls. No API trust. No external dependencies. Pure math.

### 11.4 With Proof of Satoshi

When PoS is enabled, Step 5 is inserted: 1 → 2 → 3 → 4 → **5** → 6 → **GATE** → 7 → 8.

Step 5 requires a blockchain query, introducing an external trust dependency. The strength of this check depends on the receiver’s connection to the Bitcoin network.

-----

## 12. Key Discovery

### 12.1 .well-known/butl.json

Receivers SHOULD publish their public key at a standard HTTPS endpoint following RFC 8615:

```
GET https://{domain}/.well-known/butl.json
```

### 12.2 Response Format

```json
{
  "version": 1,
  "address": "bc1q...",
  "pubkey": "02a163...",
  "pos_required": false,
  "pos_min_satoshis": 0,
  "freshness_window": 144,
  "supported_versions": [1],
  "updated": "2026-03-14T00:00:00Z"
}
```

### 12.3 Requirements

- MUST be served over HTTPS. HTTP responses MUST be rejected.
- Content-Type: `application/json`.
- Clients SHOULD cache the response (recommended TTL: 1 hour).
- Out-of-band verification is RECOMMENDED for first contact.

### 12.4 DNS TXT Alternative

For domains that cannot serve HTTP endpoints:

```
_butl.example.com  TXT  "v=1; addr=bc1q...; pubkey=02..."
```

Clients SHOULD prefer `.well-known/butl.json` over DNS TXT when both are available.

See <WELL_KNOWN_BUTL_SPEC.md> for the complete specification including multi-key organizations and IANA registration.

-----

## 13. Transport and Application Bindings

BUTL is transport-agnostic. It defines how to sign, encrypt, verify, and gate a message — not how to move the bytes. The following bindings are defined for common protocols:

**Email (SMTP).** BUTL fields are carried as X-headers (`X-BUTL-Sender`, `X-BUTL-Signature`, etc.). The encrypted payload is a MIME attachment. The receiving email client verifies the BUTL gate before rendering or saving the attachment.

**HTTP / API.** Phase 1: BUTL fields as HTTP headers. Phase 2: encrypted payload as the request or response body, transmitted after the receiver confirms verification.

**WebSocket.** Frame 1 (text): BUTL header as key-value lines. Frame 2 (text): `BUTL-READY` confirmation from server. Frame 3 (binary): encrypted payload.

**Instant Messaging.** BUTL headers form the message envelope. The encrypted payload is the message body. Address chaining provides verifiable conversation threading.

**Website Login.** The server presents a challenge (nonce + block height). The client signs the challenge with a BUTL header. The server verifies signature, freshness, and optionally balance. No passwords.

**Video/Voice.** BUTL handshake during signaling phase. ECDH-derived shared secret used as the media encryption key.

**IoT.** Each device has a BUTL keypair. Device-to-device and device-to-server communication uses BUTL headers for mutual authentication. Firmware updates are BUTL-signed — the device verifies before installing.

-----

## 14. Security Considerations

### 14.1 Cryptographic Assumptions

BUTL’s security rests on three assumptions:

1. **Elliptic Curve Discrete Logarithm Problem (ECDLP) on secp256k1** — Computing a private key from a public key is computationally infeasible. Best known classical attack: O(√n) via Pollard’s rho (~2^128 operations).
1. **Collision resistance of SHA-256** — Finding two inputs with the same hash is infeasible. Best known attack: 2^254.9 (no practical impact).
1. **Security of AES-256-GCM** — IND-CCA2 secure (indistinguishability under adaptive chosen-ciphertext attack) and INT-CTXT secure (integrity of ciphertext).

### 14.2 No New Assumptions

BUTL introduces no novel cryptographic constructions. The security of the composition follows directly from the security of the components. If any of these primitives break, the consequences extend far beyond BUTL to Bitcoin, TLS, SSH, and the majority of internet security infrastructure.

### 14.3 Attack Resistance

**Man-in-the-middle.** The ECDSA signature cannot be forged without the sender’s private key. The ECDH shared secret cannot be computed without the receiver’s private key. The AAD binding prevents payload redirection.

**Replay attacks.** Block height freshness + nonce. Captured messages expire after the freshness window.

**Key compromise.** Each message uses a fresh keypair via address chaining. Compromising one key affects one message. Ephemeral ECDH keys provide additional forward secrecy.

**Sybil attacks.** Mitigated only when Proof of Satoshi is enabled. When disabled, unlimited identities can be created at zero cost.

**Payload attacks.** Verify-before-download ensures no payload data reaches the device until all header checks pass. Malware, phishing, resource exhaustion, and supply-chain attacks are blocked at the gate.

### 14.4 Known Limitations

- **No key revocation** in v1.2. A compromised key can continue the chain. Planned for a future version.
- **Block height requires connectivity.** The receiver needs an approximate current tip (cacheable, updates ~every 10 minutes).
- **ECDSA + ECDH on same key.** Mixes two security games. Mitigated by `BUTL-EphemeralPubKey` (recommended best practice).
- **Balance check external dependency.** When PoS is enabled, the blockchain query can be spoofed by a compromised API. Running a full node provides the strongest guarantee.
- **Private key = identity.** Loss is permanent. Compromise enables impersonation. This is an inherent property of self-sovereign systems.

### 14.5 Quantum Considerations

secp256k1 ECDSA and ECDH are theoretically vulnerable to Shor’s algorithm on a fault-tolerant quantum computer. Such computers do not exist and are not expected for 10+ years. A post-quantum migration path (hybrid ECDSA + Dilithium, hybrid ECDH + ML-KEM) is planned for v2.0.

-----

## 15. Cryptographic Primitives

|Primitive       |Specification    |Usage in BUTL                                                                                                      |
|----------------|-----------------|-------------------------------------------------------------------------------------------------------------------|
|SHA-256         |FIPS 180-4       |Hashing, signing payload, payload integrity, ThreadID, chain proof hash, ECDH secret derivation, address derivation|
|Double SHA-256  |Bitcoin Protocol |Block hashing, Base58Check checksum                                                                                |
|ECDSA           |SEC 2, secp256k1 |Message signing and verification, chain proof signing and verification                                             |
|ECDH            |SEC 1, secp256k1 |Shared secret derivation for payload encryption                                                                    |
|AES-256-GCM     |NIST SP 800-38D  |Authenticated payload encryption and decryption                                                                    |
|RIPEMD-160      |ISO/IEC 10118-3  |Address derivation (second hash after SHA-256)                                                                     |
|Base58Check     |Bitcoin Protocol |P2PKH address encoding (version byte + checksum)                                                                   |
|Bech32 / Bech32m|BIP 173 / BIP 350|SegWit address encoding                                                                                            |

-----

## 16. Implementation Notes

### 16.1 Key Storage

Private keys MUST be encrypted at rest. Recommended: Argon2id password derivation + AES-256-GCM. Previous private keys SHOULD be securely deleted after chain proof creation (memory zeroed).

### 16.2 Key Derivation

BIP-32 hierarchical deterministic derivation is RECOMMENDED for production. BIP-39 mnemonic seed phrases are RECOMMENDED for identity backup.

### 16.3 Proof of Satoshi Caching

When PoS is enabled, receivers MAY cache balance results for up to 10 minutes to reduce blockchain query frequency.

### 16.4 Block Height Caching

The current block height is a single global value that changes approximately every 10 minutes. Receivers SHOULD cache it and update periodically rather than querying per-message.

### 16.5 Minimum Implementation (without PoS)

SHA-256, secp256k1 (ECDSA + ECDH), AES-256-GCM, RIPEMD-160, Base58Check or Bech32, and a cached block height value. No blockchain balance queries needed. No external service dependencies.

### 16.6 Minimum Implementation (with PoS)

All of the above plus a blockchain balance query method (Bitcoin Core RPC, Electrum, or REST API such as mempool.space).

### 16.7 Supported Versions

|Protocol Version|BUTL-Version Value|Status |
|----------------|------------------|-------|
|v1.2            |`1`               |Current|

All v1.x spec revisions (v1.0, v1.1, v1.2) use protocol version `1`. The spec revision tracks documentation changes. The protocol version tracks wire-format changes. See <VERSIONING.md> for version negotiation and backward compatibility rules.

-----

*END OF SPECIFICATION*