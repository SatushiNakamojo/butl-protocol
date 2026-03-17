# BUTL Protocol — AI Quick Start Prompt

## Compact Context for Fast AI Loading

**When to use this:** Paste this into any AI conversation when you need quick, working knowledge of BUTL without the full 18-section briefing document. This covers the essentials in under 60 lines. For complete context, use <AI_BRIEFING_DOCUMENT.md>. For code generation, use <AI_CODE_GENERATION_REFERENCE.md>.

-----

## Paste Everything Below This Line

```
BUTL (Bitcoin Universal Trust Layer) is an open cryptographic protocol that adds
identity, encryption, integrity, freshness, forward privacy, and optional sybil
resistance to any internet communication using Bitcoin's existing cryptography.

NOT a blockchain. NOT a token. A communication protocol like TLS, anchored to Bitcoin.

PRIMITIVES: SHA-256, ECDSA/ECDH on secp256k1, AES-256-GCM. Nothing novel.

IDENTITY: Private key (32 bytes) → Public key (33 bytes compressed, secp256k1)
→ Bitcoin address (SHA-256 → RIPEMD-160 → Base58Check). Self-sovereign. No CA.

8-STEP VERIFICATION GATE (verify-before-download):
  Phase 1 (header only, no payload on device):
    1. Structure check (fields present + pubkey→address consistency)
    2. Receiver match (BUTL-Receiver == my address)
    3. Signature (ECDSA over SHA-256 of canonical 10-field payload)
    4. Freshness (0 ≤ current_tip - block_height ≤ 144)
    5. Proof of Satoshi (OPTIONAL — receiver toggle + slider, 1 sat to 21M BTC)
    6. Chain proof (previous key signed SHA-256 of current address)
  ANY FAIL → drop connection, payload never sent.
  Phase 2 (payload, only if gate passes):
    7. Payload hash (SHA-256 of encrypted blob == header value)
    8. Decrypt (AES-256-GCM with ECDH shared secret, AAD = sender+receiver addrs)

ENCRYPTION: shared_secret = SHA-256((my_priv × their_pub).x). Commutative.
  Payload = IV(12) || AES-256-GCM(key=secret, aad=sender_addr||receiver_addr) || tag(16)

CANONICAL SIGNING PAYLOAD (10 fields, \n separated, fixed order):
  BUTL-Version, Sender, SenderPubKey, Receiver, ReceiverPubKey,
  BlockHeight, BlockHash, PayloadHash, PrevAddr, Nonce

ADDRESS CHAINING: New keypair per message. Chain proof links them:
  chain_proof = ECDSA_SIGN(SHA-256(new_address), previous_private_key)
  ThreadID = SHA-256_HEX(first_address_in_chain). Constant across all messages.

PROOF OF SATOSHI: Optional. Receiver sets toggle (on/off) + slider (min sats).
  Off = pure math, 99.7% confidence, zero external dependencies.
  On = blockchain query, 97% confidence, sybil resistance.

KEY DISCOVERY: https://{domain}/.well-known/butl.json
  Returns: {version, address, pubkey, pos_required, freshness_window}

ECOSYSTEM: BUTL-ID (key management, never exposes private key)
  → BUTL-Contacts (contact book + handoff engine)
  → Spoke apps (Chat, Video, FileTransfer, Login, Email, Sign)

LICENSE: MIT OR Apache 2.0 (dual, user's choice).
  PATENTS.md + Defensive Patent Pledge apply to ALL users.

IMPLEMENTATIONS:
  Python: butl_v12.py (1,146 lines, production + stub mode)
  Rust: butl_v12.rs (1,264 lines, real libsecp256k1 + AES-256-GCM)
  MVP: butl_mvp_v12.py (549 lines, zero deps, proves all 8 claims)

VERIFY: python3 mvp/butl_mvp_v12.py → "ALL 8 PROOFS PASSED"

KNOWN GAPS: No key revocation (v2.0), no multi-party (v2.0),
  not post-quantum (v2.0 hybrid mode planned), stub blockchain in impls.

v1.2 STATUS: Published. 54 files. Specification complete. Implementations verified.
```

-----

## What an AI Can Do With This Context

After reading the prompt above, an AI should be able to:

- **Explain** the BUTL protocol at any level of detail (60-second overview to deep technical)
- **Compare** BUTL to TLS, PGP, Signal, OAuth, and other trust protocols
- **Answer** questions about any aspect of the protocol (identity, encryption, freshness, chaining, PoS, gate)
- **Generate** code snippets that construct BUTL headers, canonical payloads, and chain proofs
- **Debug** issues with BUTL implementations by understanding the expected behavior of each gate step
- **Design** new integrations (API auth, email, IoT, login) following the BUTL pattern
- **Discuss** tradeoffs (PoS enabled vs disabled, MIT vs Apache 2.0, pure math vs blockchain dependency)

For deeper work (full implementation, protocol extension, security analysis), use the complete <AI_BRIEFING_DOCUMENT.md>.

For code generation with exact byte formats and algorithms, use <AI_CODE_GENERATION_REFERENCE.md>.

-----

## Usage Examples

### Quick Question

```
[Paste the prompt above]

Question: How does BUTL prevent replay attacks?
```

### Integration Design

```
[Paste the prompt above]

I want to add BUTL authentication to a REST API built with FastAPI.
Show me how the server verifies incoming requests.
```

### Code Review

```
[Paste the prompt above]

Here's my BUTL implementation in Go. Can you check if my canonical
signing payload construction matches the spec?

[paste code]
```

### Comparison Analysis

```
[Paste the prompt above]

Compare BUTL's trust model to mTLS (mutual TLS). What does each
provide that the other doesn't?
```

-----

*56 lines of context. Enough for any AI to understand, discuss, and work with the BUTL protocol. For the full picture, use the briefing document.*