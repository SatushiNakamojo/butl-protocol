# BUTL MVP Quick Reference

## One-Page Command Card

-----

## Run the Proof

**Python (zero dependencies):**

```bash
python3 butl_mvp_v12.py
```

**Rust (real secp256k1 + AES-256-GCM):**

```bash
cargo new butl-mvp && cd butl-mvp
# Copy butl_mvp_v12.rs → src/main.rs
# Copy Cargo.toml from mvp/ folder
cargo run --release
```

-----

## What to Look For

|Line in Output                                   |What It Proves                            |
|-------------------------------------------------|------------------------------------------|
|`Original hash: ... Tampered hash: ... Match: NO`|SHA-256 detects tampering                 |
|`Address: 1...` + `Consistency check: PASS`      |Real Bitcoin address, pubkey matches      |
|`Correct key: VALID` + `Wrong key: INVALID`      |ECDSA signatures are unforgeable          |
|`Age 0: ACCEPTED` + `Age 1412: REJECTED`         |Block height freshness rejects replays    |
|`PoS DISABLED → Gate: OPEN`                      |Pure math mode works without blockchain   |
|`PoS >= 1,000 sat → PASS`                        |Balance above threshold passes            |
|`PoS >= 100,000 sat → FAIL`                      |Balance below threshold rejected          |
|`PoS zero balance → FAIL`                        |Zero balance rejected                     |
|`Chain proof: VALID` + `Forged: INVALID`         |Address chaining verified                 |
|`Shared secrets match: YES`                      |ECDH commutativity confirmed              |
|`Receiver decrypts: "Hello..."`                  |Encryption works                          |
|`Wrong receiver: AUTH FAILED`                    |Only intended receiver decrypts           |
|`Gate: OPEN` → `Steps 7-8: PASS`                 |Full gate verified                        |
|`Tampered payload: AUTH FAILED`                  |Tamper detection works                    |
|`Wrong receiver: Gate CLOSED at Step 2`          |Wrong receiver blocked, payload never sent|
|**`ALL 8 PROOFS PASSED`**                        |**Protocol verified**                     |

-----

## The 8 Steps

```
Phase 1: HEADER ONLY (no payload on device)

  Step 1  Structure check       All required fields present + pubkey→address match
  Step 2  Receiver match        Message is addressed to this device
  Step 3  Signature (ECDSA)     Sender controls the claimed key
  Step 4  Freshness             Block height within window (default: 144 ≈ 24h)
  Step 5  Proof of Satoshi      OPTIONAL — balance ≥ threshold (receiver controls)
  Step 6  Chain proof           Same sender as previous message

  ─── GATE: OPEN or CLOSED ─────────────────────────────────

Phase 2: PAYLOAD (only if gate opens)

  Step 7  Payload hash          SHA-256 of encrypted payload matches header
  Step 8  Decryption            AES-256-GCM with ECDH shared secret
```

-----

## Proof of Satoshi Scenarios

|Scenario|PoS Toggle|Threshold  |Sender Balance|Result               |
|--------|----------|-----------|--------------|---------------------|
|A       |OFF       |—          |—             |OPEN (pure math)     |
|B       |ON        |1,000 sat  |50,000 sat    |OPEN (passes)        |
|C       |ON        |100,000 sat|50,000 sat    |CLOSED (insufficient)|
|D       |ON        |1 sat      |0 sat         |CLOSED (zero balance)|

-----

## The Three Mathematical Facts

Everything in BUTL depends on three facts:

1. **SHA-256 is collision-resistant** — different inputs produce different outputs
1. **secp256k1 ECDLP is hard** — knowing the public key doesn’t reveal the private key
1. **ECDH is commutative** — `(a × G) × b == (b × G) × a` on the curve

These secure Bitcoin ($1T+), TLS, and SSH. BUTL uses nothing else.

-----

## Files in This Folder

|File                        |What It Does                                      |
|----------------------------|--------------------------------------------------|
|`butl_mvp_v12.py`           |Python MVP — zero dependencies, run with `python3`|
|`butl_mvp_v12.rs`           |Rust MVP — real libsecp256k1 + AES-256-GCM        |
|`Cargo.toml`                |Rust dependencies for the MVP                     |
|`HOW_TO_PROVE_BUTL_WORKS.md`|Step-by-step beginner guide                       |
|`VERIFICATION_CHECKLIST.md` |Printable checkbox form                           |
|`QUICK_REFERENCE.md`        |This card                                         |

-----

## Success Criteria

If the output ends with:

```
ALL 8 PROOFS PASSED
```

The protocol works. On your machine. Verified by math.

-----

*One file. One command. Eight proofs. Print this card and keep it next to your terminal.*