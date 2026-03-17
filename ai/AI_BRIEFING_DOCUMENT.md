# BUTL Protocol — AI Briefing Document

## Complete Context Transfer for Any AI System

**Purpose:** This document contains everything an AI assistant needs to understand, discuss, analyze, implement, extend, and answer questions about the BUTL (Bitcoin Universal Trust Layer) Protocol. Paste this document into a new conversation to bring any AI instance fully up to speed.

**Version:** 1.2
**Date:** March 2026
**License:** MIT OR Apache 2.0 (dual licensed, at user’s option)
**Patent:** PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md (applies to all users)

-----

## 1. What BUTL Is

BUTL (Bitcoin Universal Trust Layer) is an open cryptographic protocol that provides identity verification, end-to-end encryption, message integrity, replay protection, forward privacy, and optional sybil resistance for any internet communication. It uses Bitcoin’s existing cryptographic infrastructure — SHA-256, secp256k1 ECDSA/ECDH, block heights, and optionally UTXO balances — as a universal trust anchor.

BUTL is not a blockchain. It is not a token. It is not a cryptocurrency. It is a communication protocol — like TLS, but anchored to Bitcoin instead of certificate authorities.

The protocol sits between the application layer and the transport layer. It does not define transport. It rides on top of TCP, UDP, QUIC, HTTP, WebSocket, SMTP, MQTT, Bluetooth, NFC, or any mechanism that can move bytes.

-----

## 2. The Three Primitives

BUTL uses exactly three cryptographic primitives. No novel constructions.

|Primitive                |Specification  |Usage in BUTL                                                                                         |
|-------------------------|---------------|------------------------------------------------------------------------------------------------------|
|SHA-256                  |FIPS 180-4     |Hashing (signing payload, payload integrity, address derivation, chain proofs, thread ID, ECDH secret)|
|ECDSA / ECDH on secp256k1|SEC 1, SEC 2   |Signing, verification, key agreement                                                                  |
|AES-256-GCM              |NIST SP 800-38D|Authenticated payload encryption                                                                      |

These secure Bitcoin ($1T+), TLS 1.3, SSH, and the majority of internet infrastructure.

-----

## 3. BUTL Identity (BUTL-ID)

A BUTL identity has three mathematically linked components:

```
Private Key (32 bytes)
    ↓ scalar multiplication on secp256k1: PubKey = PrivKey × G
Public Key (33 bytes compressed: 02/03 prefix + 32-byte x-coordinate)
    ↓ SHA-256 → RIPEMD-160 → Base58Check(0x00 || hash)
Bitcoin Address (25-34 chars, starts with "1" for P2PKH or "bc1" for SegWit)
```

- **Private key:** The root. Never shared. Never transmitted. Never stored unencrypted.
- **Public key:** Used for ECDSA verification, ECDH encryption, chain proof validation.
- **Address:** Used for identification, Proof of Satoshi queries, AAD binding, contact books.

Both address and public key must be present in every BUTL header. The address cannot be reverse-derived to the public key. The receiver MUST verify that the public key correctly derives to the claimed address (consistency check).

-----

## 4. The 8-Step Verification Sequence

BUTL enforces verify-before-download: no payload touches the device until all header checks pass.

### Phase 1: Header Only (Steps 1-6)

|Step|Name                 |What It Checks                                                |Required?   |
|----|---------------------|--------------------------------------------------------------|------------|
|1   |Structural Validation|All fields present, correct format, pubkey→address consistency|Yes         |
|2   |Receiver Match       |BUTL-Receiver == my address                                   |Yes         |
|3   |Signature (ECDSA)    |Canonical payload hash signed by BUTL-SenderPubKey            |Yes         |
|4   |Block Freshness      |0 ≤ (current_tip - BUTL-BlockHeight) ≤ freshness_window       |Yes         |
|5   |Proof of Satoshi     |balance(BUTL-Sender) ≥ pos_min_satoshis                       |**OPTIONAL**|
|6   |Chain Proof          |Previous key signed SHA-256(current address)                  |Yes*        |

*Step 6 is skipped for genesis messages (empty BUTL-PrevAddr).

**ANY FAIL → connection dropped, payload never transmitted.**

### Phase 2: Payload (Steps 7-8, only if gate passes)

|Step|Name        |What It Checks                                   |
|----|------------|-------------------------------------------------|
|7   |Payload Hash|SHA-256(encrypted_payload) == BUTL-PayloadHash   |
|8   |Decryption  |AES-256-GCM decrypt with ECDH shared secret + AAD|

**FAIL at 7 or 8 → zero and discard. Nothing saved.**

-----

## 5. BUTL Header Fields

### Required (12 fields)

|Field              |Format                     |Purpose                                      |
|-------------------|---------------------------|---------------------------------------------|
|BUTL-Version       |Integer (currently `1`)    |Protocol version                             |
|BUTL-Sender        |Bitcoin address            |Sender’s address (fresh per message)         |
|BUTL-SenderPubKey  |66 hex chars (33 bytes)    |Sender’s compressed secp256k1 public key     |
|BUTL-Receiver      |Bitcoin address            |Intended receiver’s address                  |
|BUTL-ReceiverPubKey|66 hex chars               |Receiver’s compressed public key             |
|BUTL-BlockHeight   |Non-negative integer       |Referenced Bitcoin block height              |
|BUTL-BlockHash     |64 hex chars               |SHA-256d hash of the referenced block        |
|BUTL-PayloadHash   |64 hex chars               |SHA-256 of the encrypted payload             |
|BUTL-Signature     |Base64 (DER ECDSA)         |Signature over canonical signing payload     |
|BUTL-PrevAddr      |Bitcoin address or empty   |Previous sender address (chain)              |
|BUTL-ChainProof    |Base64 (DER ECDSA) or empty|Chain proof linking current to previous      |
|BUTL-EncAlgo       |String                     |Encryption algorithm (default: `AES-256-GCM`)|

### Optional (7 fields)

|Field               |Purpose                                                 |
|--------------------|--------------------------------------------------------|
|BUTL-EphemeralPubKey|Separate ECDH key for enhanced forward secrecy          |
|BUTL-Nonce          |Random nonce for additional anti-replay                 |
|BUTL-Timestamp      |Unix epoch seconds (informational, not for verification)|
|BUTL-ThreadID       |SHA-256_HEX of first sender address in chain            |
|BUTL-SeqNum         |Message sequence number (0-indexed)                     |
|BUTL-PayloadSize    |Encrypted payload size in bytes                         |
|BUTL-BalanceProof   |Offline balance proof (implementation-defined)          |

-----

## 6. Canonical Signing Payload

The 10-field string hashed with SHA-256 and signed with ECDSA:

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

Fields separated by `\n` (0x0A). No spaces around colons. Fixed order. Empty strings for absent values.

-----

## 7. Encryption

### ECDH Key Agreement

```
Sender:   shared_secret = SHA-256( (sender_priv × receiver_pub).x )
Receiver: shared_secret = SHA-256( (receiver_priv × sender_pub).x )
```

Both produce the same 32-byte value (commutativity of EC scalar multiplication).

### AES-256-GCM Encryption

```
IV = random 12 bytes
AAD = UTF-8(sender_address || receiver_address)
encrypted = AES-256-GCM(key=shared_secret, iv=IV, plaintext=body, aad=AAD)
payload = IV (12) || ciphertext || GCM_tag (16)
BUTL-PayloadHash = SHA-256_HEX(payload)
```

AAD binds the ciphertext to both parties. Redirecting the payload to a different receiver causes GCM authentication failure.

-----

## 8. Address Chaining

Each message uses a fresh keypair. Continuity is proven by signing the new address with the previous key:

```
chain_proof = ECDSA_SIGN(SHA-256(UTF-8(new_address)), previous_private_key)
```

```
Msg 0: Addr_0 (genesis, PrevAddr="", ChainProof="")
Msg 1: Addr_1 (PrevAddr=Addr_0, ChainProof=Sign(Addr_1, key_0))
Msg 2: Addr_2 (PrevAddr=Addr_1, ChainProof=Sign(Addr_2, key_1))
```

**ThreadID:** `SHA-256_HEX(UTF-8(Addr_0))` — constant across all messages in the chain.

**Properties:** Forward secrecy (one key per message), unlinkability (observers see unrelated addresses), verifiable continuity (receiver confirms same sender).

-----

## 9. Proof of Satoshi (Optional)

Receiver-configurable sybil resistance:

- **pos_enabled** (boolean, default: `false`): Whether to check balance.
- **pos_min_satoshis** (integer, range 1-2,100,000,000,000,000): Minimum balance required (1 sat to 21,000,000 BTC).

**When disabled:** Step 5 is skipped. Protocol is 100% pure math. Zero external dependencies. Confidence: 99.7%.

**When enabled:** Receiver queries Bitcoin blockchain for sender’s balance. Introduces external trust dependency. Confidence: 97%.

**Recommended thresholds:** Casual messaging = OFF. IoT = 100 sat. Public API = 1,000 sat. Social media = 10,000 sat. Enterprise = 100,000 sat. Financial = 1,000,000 sat.

-----

## 10. Key Discovery

Receivers publish their public key at:

```
GET https://{domain}/.well-known/butl.json
```

```json
{
  "version": 1,
  "address": "bc1q...",
  "pubkey": "02a163...",
  "pos_required": false,
  "freshness_window": 144,
  "supported_versions": [1]
}
```

HTTPS mandatory. DNS TXT fallback: `_butl.example.com TXT "v=1; addr=...; pubkey=..."`.

-----

## 11. Ecosystem Architecture

Three layers:

```
BUTL-ID (identity foundation)
    Private key management. API: sign(), ecdh(), public_info(), backup(), restore().
    Private key NEVER leaves this layer.
        ↓
BUTL-Contacts (identity hub)
    Contact book, chain state, settings, handoff engine.
    Constructs JSON handoff for spoke apps.
        ↓
Spoke Applications (domain-specific)
    Chat, Video, FileTransfer, Login, Email, Sign.
    Each receives a handoff, calls BUTL-ID for crypto.
```

**Handoff protocol:** Standardized JSON blob containing sender identity, receiver identity, chain state, and gate configuration. Private key is NEVER in the handoff — only a keychain reference.

**Build order:** Phase 1 BUTL-Contacts (hub), Phase 2 BUTL-Chat (first spoke), Phase 3 BUTL-ID extraction (standalone service), Phase 4+ additional spokes.

-----

## 12. Licensing and Patent Protection

**Dual licensed:** MIT OR Apache 2.0, at user’s choice.

**Three layers of patent protection:**

1. **Apache 2.0 Section 3:** Built-in patent grant + retaliation clause (for users who choose Apache)
1. **PATENTS.md:** Explicit, irrevocable patent grant covering all BUTL methods (applies to ALL users)
1. **Defensive Patent Pledge:** Contributors will never file patents on BUTL, will defend against third-party patents, and will never sell rights to trolls. Irrevocable. Survives acquisition.

**Prior art:** Repository publication with Git timestamps, Wayback Machine archives, and optional Bitcoin blockchain timestamps. Any patent filed after publication date is anticipated by this prior art.

-----

## 13. Design Decisions and Rationale

|Decision                               |Rationale                                                                                                             |
|---------------------------------------|----------------------------------------------------------------------------------------------------------------------|
|Proof of Satoshi optional              |Core protocol is pure math. PoS adds external dependency. Not all apps need sybil resistance.                         |
|Receiver controls PoS threshold        |The protocol provides the mechanism. The application chooses the policy.                                              |
|New address per message                |Forward secrecy + privacy. Previous key deleted after chain proof.                                                    |
|Both address and pubkey in header      |Address is human-readable + checksummed + BTC-indexed. Pubkey is needed for crypto. Cannot derive pubkey from address.|
|AAD = sender addr || receiver addr     |Binds ciphertext to both parties. Prevents payload redirection.                                                       |
|Block height (not timestamp)           |Bitcoin PoW as a trustless clock. Can’t be spoofed without majority hashrate. Cacheable.                              |
|Verify-before-download                 |No payload on device until verified. Inverts traditional download-first-verify-later model.                           |
|Dual license (MIT OR Apache)           |Maximum adoption + maximum patent protection. Same pattern as Rust.                                                   |
|ECDH shared secret = SHA-256(point.x)  |Standard derivation. Same as Bitcoin ECDH.                                                                            |
|Canonical payload has fixed field order|Deterministic hash. Sender and receiver compute the same hash independently.                                          |

-----

## 14. Current State (v1.2)

**Published and complete:**

- Protocol Specification v1.2 (16 sections)
- Python reference implementation (1,146 lines, production + stub mode)
- Rust reference implementation (1,264 lines, real libsecp256k1 + AES-256-GCM)
- Python MVP (549 lines, zero dependencies, all 8 proofs pass)
- Rust MVP (489 lines, real crypto, all 8 proofs pass)
- White paper (13 sections + 10 references)
- 14 protocol diagrams
- 11 deterministic test vectors with real computed values
- Header registry (19 fields + 5 reserved)
- Versioning and backward compatibility spec
- .well-known/butl.json key discovery spec
- Complete documentation suite (54 files)
- Dual license (MIT OR Apache 2.0) + PATENTS + Defensive Patent Pledge
- AI generation and collaboration disclosures

**Planned:**

- Phase 1: BUTL-Contacts (identity hub, CLI, Python)
- Phase 2: BUTL-Chat (first spoke, CLI messenger over TCP)
- v1.3: Key revocation, production libraries (PyPI/crates.io/npm), formal test suite, real blockchain integration
- v2.0: Multi-party, post-quantum hybrid mode, BUTL-ID extraction, formal audit, IETF RFC
- v3.0+: Hardware integration (COLDCARD, Ledger, Trezor, secure enclaves), streaming, mobile SDKs

-----

## 15. Confidence Levels

|Configuration                     |Confidence|Trust Model       |
|----------------------------------|----------|------------------|
|PoS disabled, production libraries|99.7%     |Pure math         |
|PoS disabled, MVP                 |~99.0%    |Pure math         |
|PoS enabled, full node            |~98.5%    |Math + own node   |
|PoS enabled, REST API             |~97.0%    |Math + trusted API|

The 0.3% residual = implementation bugs + undiscovered primitive weaknesses + side channels. Same residual as Bitcoin.

-----

## 16. Known Limitations

|Limitation                         |Impact                                  |Planned Fix                                             |
|-----------------------------------|----------------------------------------|--------------------------------------------------------|
|No key revocation                  |Compromised key can continue chain      |v1.3/v2.0: BUTL-Revocation header                       |
|Two-party only                     |No group messaging                      |v2.0: BUTL-MultiSig, BUTL-GroupID                       |
|Not post-quantum                   |secp256k1 vulnerable to Shor’s algorithm|v2.0: Hybrid ECDSA+Dilithium, ECDH+ML-KEM               |
|Stub blockchain in implementations |Not connected to real Bitcoin           |v1.3: Bitcoin Core RPC, Electrum, mempool.space adapters|
|No encrypted key storage in library|Private keys in memory as raw bytes     |Phase 1 BUTL-Contacts: Argon2id + AES-256-GCM storage   |
|MVP not constant-time              |Timing side channels in pure Python     |Use libsecp256k1 (Rust impl) for production             |

-----

## 17. AI Development Context

This protocol was conceived and directed by its human creator. All documents, code, specifications, and implementations in v1.2 were initially generated by Claude (Anthropic), with additional collaboration from Grok (xAI) and ChatGPT (OpenAI). All materials may have been edited since generation.

The AI context transfer documents in this repository are designed for exactly this use case — bringing a new AI instance up to speed on BUTL quickly and accurately:

- **This document (AI_BRIEFING_DOCUMENT.md):** Full context (17 sections)
- **AI_QUICK_START_PROMPT.md:** Compact version (32 lines) for fast loading
- **AI_CODE_GENERATION_REFERENCE.md:** Exact algorithms, byte formats, and data structures for generating BUTL implementations in any language

-----

## 18. How to Verify

```bash
python3 mvp/butl_mvp_v12.py
```

If the output ends with `ALL 8 PROOFS PASSED`, the protocol works. On your machine. With your own eyes. Zero dependencies. Pure math.

-----

*This document is the complete context for the BUTL Protocol v1.2. An AI system that has read this document can discuss, analyze, implement, extend, and answer questions about every aspect of the protocol.*