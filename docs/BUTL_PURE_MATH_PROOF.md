# BUTL Without Proof of Satoshi: 100% Pure Mathematics

## Why the Core Protocol Has Zero External Trust Dependencies

-----

## The Claim

When Proof of Satoshi is disabled, every operation in the BUTL protocol is a deterministic mathematical computation performed locally on data contained entirely within the message header and payload. No network calls. No API queries. No third-party trust. No oracles. No external state beyond a single cached integer (the Bitcoin block height).

The protocol’s security derives entirely from three mathematical hardness assumptions that have been scrutinized by the global cryptographic community for decades and collectively secure the majority of the internet’s critical infrastructure.

This document proves that claim by walking through each verification step, identifying the exact computation, the exact inputs, the security primitive it relies on, and whether any external dependency is involved.

-----

## The Three Mathematical Foundations

Before examining the steps, here are the three facts that everything rests on:

### 1. The Elliptic Curve Discrete Logarithm Problem (ECDLP) on secp256k1

Given a generator point G and a public key Q = k × G on the secp256k1 curve, computing k (the private key) from Q and G is computationally infeasible. The best known classical attack is Pollard’s rho algorithm, which requires approximately 2^128 operations — a number larger than the estimated number of atoms in the observable universe.

**What this secures in BUTL:** Identity (you can’t forge someone’s keypair), signatures (you can’t sign without the private key), and encryption (you can’t compute the ECDH shared secret without a private key).

**What else this secures:** Every Bitcoin transaction ever made. The Ethereum network. Hundreds of billions of dollars in daily cryptographic operations.

### 2. The Collision Resistance of SHA-256

Finding two different inputs that produce the same SHA-256 output requires approximately 2^128 operations (birthday attack). Finding a specific input that matches a given hash (preimage attack) requires approximately 2^256 operations.

**What this secures in BUTL:** Message integrity (BUTL-PayloadHash), canonical signing payload construction, address derivation, chain proof hashing, thread identity, and ECDH shared secret derivation.

**What else this secures:** Bitcoin mining (proof of work), TLS certificate chains, SSH session integrity, Git commit integrity, PGP signatures.

### 3. The Security of AES-256-GCM

AES-256-GCM provides IND-CCA2 security (indistinguishability under adaptive chosen-ciphertext attack) and INT-CTXT security (integrity of ciphertext). No practical attack is known against AES-256 with a random key.

**What this secures in BUTL:** Payload confidentiality (only the intended receiver reads the message) and payload authenticity (any modification to the ciphertext is detected by the GCM tag).

**What else this secures:** TLS 1.3 (the protocol securing HTTPS), IPsec VPNs, SSH sessions, every major encrypted messaging protocol.

-----

## Step-by-Step Proof: No External Dependencies

### Step 1: Structural Validation

**What happens:** The receiver checks that all required header fields are present, correctly formatted, and internally consistent. Specifically:

- `BUTL-Version` is a supported integer
- `BUTL-SenderPubKey` is 66 hex characters (33 bytes compressed, starting with `02` or `03`)
- `BUTL-BlockHash` is 64 hex characters
- `BUTL-PayloadHash` is 64 hex characters
- `BUTL-Signature` is present and non-empty
- `BUTL-SenderPubKey` correctly derives to `BUTL-Sender` (consistency check)

**Computation:** String comparison, length checking, hex validation, and the consistency check: `Base58Check(0x00 || RIPEMD-160(SHA-256(pubkey)))` compared to the claimed address.

**Security primitive:** SHA-256, RIPEMD-160, Base58Check (all deterministic hash functions).

**External dependency:** None. All inputs are in the header. All computations are local.

-----

### Step 2: Receiver Match

**What happens:** The receiver compares `BUTL-Receiver` to its own Bitcoin address.

**Computation:** String equality check.

**Security primitive:** None — this is a simple comparison.

**External dependency:** None. The receiver knows its own address.

-----

### Step 3: Signature Verification (ECDSA)

**What happens:** The receiver reconstructs the canonical signing payload from header fields, hashes it with SHA-256, and verifies the ECDSA signature against the sender’s public key.

**Computation:**

```
1. canonical = "BUTL-Version:{v}\nBUTL-Sender:{s}\n..." (10 fields, fixed order)
2. message_hash = SHA-256(UTF-8(canonical))
3. valid = ECDSA_VERIFY(BUTL-SenderPubKey, BUTL-Signature, message_hash)
```

ECDSA verification involves:

- Parsing the DER-encoded signature into (r, s)
- Computing w = s⁻¹ mod n
- Computing u₁ = (message_hash × w) mod n
- Computing u₂ = (r × w) mod n
- Computing the point R = u₁ × G + u₂ × Q (where Q is the sender’s public key)
- Checking that R.x mod n == r

**Security primitive:** SHA-256 (hashing) + ECDSA on secp256k1 (signature verification).

**External dependency:** None. The public key is in the header. The signature is in the header. The canonical payload is reconstructed from header fields. Every operation is a local computation on secp256k1.

-----

### Step 4: Block Freshness

**What happens:** The receiver checks that the referenced block height is within the freshness window.

**Computation:**

```
age = current_chain_tip - BUTL-BlockHeight
valid = (0 ≤ age ≤ freshness_window)
```

**Security primitive:** Integer comparison.

**External dependency:** The `current_chain_tip` is a single integer — the current Bitcoin block height. This number changes approximately every 10 minutes and can be cached locally. Once cached, the freshness check is pure integer arithmetic with no network call.

**Note:** The block height cache is the one external piece of information in pure math mode. It is a single globally known integer (not a secret, not per-message, not per-sender) that can be obtained once and reused for hundreds of messages. Caching it for 10-60 minutes is standard practice. During that window, all freshness checks are local math.

**Optional verification:** `BUTL-BlockHash` can also be verified against a cached block hash. This is an additional check but not strictly required — the block height integer alone provides replay protection.

-----

### Step 5: Proof of Satoshi

**SKIPPED** when `pos_enabled = false`.

This step does not execute. No blockchain query is made. No external service is contacted. The gate proceeds directly from Step 4 to Step 6.

This is the only step in the entire protocol that requires a real-time external query when enabled. By making it optional, the core protocol achieves zero external dependencies.

-----

### Step 6: Chain Proof Verification

**What happens:** For threaded messages (non-genesis), the receiver verifies that the current sender address was signed by the previous sender’s private key.

**Computation:**

```
1. chain_hash = SHA-256(UTF-8(BUTL-Sender))
2. valid = ECDSA_VERIFY(previous_sender_pubkey, BUTL-ChainProof, chain_hash)
```

**Security primitive:** SHA-256 (hashing) + ECDSA on secp256k1 (signature verification).

**External dependency:** None. The previous sender’s public key was stored locally from the previous verified message. The current address and chain proof are in the header. All computations are local.

**For genesis messages:** `BUTL-PrevAddr` is empty. This step is skipped entirely. No computation needed.

-----

### ── GATE BOUNDARY ──

If Steps 1-4 and 6 all pass, the gate is OPEN. The payload is now requested.

Everything up to this point involved only the header — no payload data touched the device.

-----

### Step 7: Payload Hash Verification

**What happens:** The receiver computes the SHA-256 hash of the received encrypted payload and compares it to `BUTL-PayloadHash` from the header.

**Computation:**

```
computed = SHA-256(encrypted_payload)
valid = (computed == BUTL-PayloadHash)
```

**Security primitive:** SHA-256 (collision resistance — an attacker cannot create a different payload with the same hash).

**External dependency:** None. The hash is computed locally on the received bytes. The expected value is in the header (which was already signature-verified in Step 3).

-----

### Step 8: Decryption (ECDH + AES-256-GCM)

**What happens:** The receiver computes the ECDH shared secret and decrypts the payload.

**Computation:**

```
1. ecdh_pubkey = BUTL-EphemeralPubKey (if present) or BUTL-SenderPubKey
2. shared_secret = SHA-256( (receiver_private_key × ecdh_pubkey).x )
3. aad = UTF-8(BUTL-Sender || BUTL-Receiver)
4. plaintext = AES-256-GCM-DECRYPT(
       key = shared_secret,
       nonce = encrypted_payload[0:12],
       ciphertext_and_tag = encrypted_payload[12:],
       aad = aad
   )
```

ECDH involves:

- Decompressing the sender’s public key to a curve point
- Performing scalar multiplication: receiver_private_key × sender_public_point
- Hashing the x-coordinate: SHA-256(shared_point.x)

AES-256-GCM decryption involves:

- Initializing AES-256 with the shared secret as the key
- Running GCM decryption with the nonce and AAD
- Verifying the 16-byte GCM authentication tag
- If the tag matches: returning the plaintext
- If the tag doesn’t match: rejecting (tampered or wrong key)

**Security primitive:** ECDH on secp256k1 (key agreement) + SHA-256 (secret derivation) + AES-256-GCM (authenticated decryption).

**External dependency:** None. The receiver’s private key is local. The sender’s public key is in the header. The encrypted payload was received in Phase 2. Every operation is local.

-----

## Summary: External Dependencies by Step

|Step               |Computation                         |External Query?    |Primitive                           |
|-------------------|------------------------------------|-------------------|------------------------------------|
|1. Structure       |String checks + SHA-256 + RIPEMD-160|**No**             |SHA-256, RIPEMD-160                 |
|2. Receiver Match  |String comparison                   |**No**             |None                                |
|3. Signature       |SHA-256 + ECDSA verify              |**No**             |SHA-256, ECDSA/secp256k1            |
|4. Freshness       |Integer comparison                  |**No** (cached tip)|None                                |
|5. Proof of Satoshi|Blockchain balance query            |**SKIPPED**        |—                                   |
|6. Chain Proof     |SHA-256 + ECDSA verify              |**No**             |SHA-256, ECDSA/secp256k1            |
|7. Payload Hash    |SHA-256 comparison                  |**No**             |SHA-256                             |
|8. Decryption      |ECDH + AES-256-GCM                  |**No**             |ECDH/secp256k1, SHA-256, AES-256-GCM|

**Total external queries when PoS is disabled: zero.**

Every step that executes is pure mathematics on data contained in the message header, the message payload, and the receiver’s own keypair.

-----

## What “Pure Math” Means Precisely

A verification step is “pure math” if and only if:

1. **All inputs are local.** The inputs come from the message (header + payload) or the receiver’s own stored data (keypair, previous sender pubkey, cached block height). No network query is needed at verification time.
1. **The computation is deterministic.** Given the same inputs, the computation always produces the same output. There is no randomness, no timing dependency, no external state.
1. **The security relies only on mathematical hardness assumptions.** The step cannot be defeated without breaking ECDLP on secp256k1, finding SHA-256 collisions, or breaking AES-256-GCM — feats that would simultaneously break Bitcoin, TLS, and most internet security.
1. **No trust is placed in any third party.** No certificate authority, no identity provider, no API server, no relay, no DNS resolver, no ISP, and no other human or organization is trusted.

Steps 1, 2, 3, 4 (with cached tip), 6, 7, and 8 satisfy all four conditions.

Step 5 (Proof of Satoshi), when enabled, violates conditions 1 and 4 — it queries a blockchain data source and trusts that source to return an accurate balance. This is why it is optional.

-----

## The Block Height Question

The one nuance in the “pure math” claim is the block height in Step 4. Strictly speaking, the receiver must know the approximate current Bitcoin block height to check freshness. This is a single publicly known integer that changes every ~10 minutes.

**Why this does not compromise the pure-math property:**

- It is a single global value, not per-sender or per-message.
- It is publicly observable by anyone running a Bitcoin node, querying any block explorer, or even reading a news article.
- It can be cached for extended periods (10 minutes to hours) without security impact — the freshness window is 144 blocks (~24 hours) by default.
- It does not require trusting any specific party. The value is independently verifiable by thousands of nodes.
- It contains no secret information. It cannot be used to compromise any key or decrypt any message.
- In offline scenarios, the receiver can use their last known block height with a wider freshness window.

Compare this to TLS, which requires trusting a specific certificate authority to vouch for the identity of the server. BUTL requires knowing a single public integer. The trust models are not comparable.

-----

## What This Property Means in Practice

### Offline Operation

Because verification is pure math, BUTL works offline. A receiver with a cached block height can verify messages without any network connectivity. This enables delay-tolerant networks, air-gapped systems, mesh networks, and environments where internet access is intermittent or unavailable.

### No Single Point of Failure

There is no server that must be online for verification to work. No CA that must be trusted. No identity provider that can revoke access. If the internet went down entirely and two devices could exchange bytes (Bluetooth, USB drive, physical paper), BUTL verification would still work.

### Censorship Resistance

No intermediary can prevent verification. A government cannot block a certificate revocation check because there is no certificate. An ISP cannot intercept a key server query because there is no key server. The math runs locally and cannot be interfered with.

### Auditability

Every step is transparent. A developer can read the verification code, identify the exact mathematical operation at each step, and confirm that no hidden network calls, no telemetry, no external lookups, and no trust assumptions are embedded. The MVP proves this in 549 lines of Python with zero dependencies.

-----

## Comparison to Other Trust Models

|Property                       |BUTL (PoS off)|TLS                   |PGP                               |OAuth                  |
|-------------------------------|--------------|----------------------|----------------------------------|-----------------------|
|External trust required        |**No**        |Yes (CA)              |No (but key exchange is a problem)|Yes (identity provider)|
|Network query at verification  |**No**        |Yes (OCSP)            |No                                |Yes (token validation) |
|Single point of failure        |**None**      |CA                    |None                              |Identity provider      |
|Works fully offline            |**Yes**       |No                    |Yes                               |No                     |
|Can be censored at verification|**No**        |Yes (CA/OCSP blocking)|No                                |Yes (provider blocking)|
|Trust assumption               |Math only     |Math + institution    |Math only                         |Math + institution     |

BUTL without Proof of Satoshi achieves the same trust model as PGP (math only, no institutions) while adding freshness (which PGP lacks), encryption (which PGP has but differently), verify-before-download (which PGP lacks entirely), and address chaining (which PGP lacks entirely).

-----

## The Confidence Number: 99.7%

When Proof of Satoshi is disabled, the protocol’s confidence is assessed at 99.7%. The 0.3% residual accounts for:

- **Implementation bugs** (any software can have bugs, regardless of protocol correctness)
- **Undiscovered weaknesses in secp256k1, SHA-256, or AES-256-GCM** (extremely unlikely given decades of scrutiny, but theoretically possible)
- **Side-channel attacks on specific implementations** (timing, power analysis — protocol-level design is correct, but implementation quality matters)

The 0.3% is not a weakness in BUTL’s design. It is the universal risk that applies to every cryptographic system ever built, including Bitcoin itself.

See <CONFIDENCE_WITHOUT_POS.md> for the full analysis.

-----

## How to Verify This Claim Yourself

```bash
python3 butl_mvp_v12.py
```

The MVP implements every step described in this document. It runs entirely offline, with zero dependencies, using pure Python implementations of secp256k1, SHA-256, ECDSA, ECDH, and authenticated encryption.

If it says `ALL 8 PROOFS PASSED`, you have verified — on your own machine, with your own eyes — that the protocol works as pure math.

-----

*When Proof of Satoshi is off, BUTL is math. Not trust. Not infrastructure. Not institutions. Math.*