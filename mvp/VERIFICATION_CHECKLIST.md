# BUTL Protocol Verification Checklist

## Print This Page and Check Off Each Item as You Verify It

This checklist records the results of running the BUTL MVP (`butl_mvp_v12.py` or `butl_mvp_v12.rs`). Each checkbox corresponds to a specific claim proven by the output. Go through the output line by line and check each item when you confirm it with your own eyes.

-----

**Date:** ___________________________

**Verified by:** ___________________________

**Computer / OS:** ___________________________

**Python version:** ___________________________

**File used:** `butl_mvp_v12.py` / `butl_mvp_v12.rs` (circle one)

-----

## Setup

- [ ] Python 3.8+ (or Rust) is installed and working
- [ ] `butl_mvp_v12.py` (or `.rs`) is saved on this computer
- [ ] No internet connection is required (can run in airplane mode)
- [ ] No external dependencies were installed (Python MVP is zero-dependency)
- [ ] The command `python3 butl_mvp_v12.py` (or `cargo run --release`) was executed

-----

## Proof 1: SHA-256 Body Integrity

- [ ] An original hash was displayed (64 hex characters)
- [ ] A tampered hash was displayed (64 hex characters, different from original)
- [ ] The two hashes do NOT match
- [ ] Output says: “PROVED: Any modification to the body changes the hash.”

**What this proves:** If anyone changes even one byte of a BUTL message in transit, the hash changes and the tampering is detected.

-----

## Proof 2: Bitcoin Address Identity

- [ ] A Bitcoin address was displayed (starts with “1”, 25-34 characters)
- [ ] A public key was displayed (starts with “02” or “03”, 66 hex characters)
- [ ] Consistency check: PASS (pubkey derives to address)
- [ ] Output says: “PROVED: Valid Bitcoin P2PKH address from secp256k1.”

**What this proves:** BUTL generates real Bitcoin addresses from real secp256k1 elliptic curve math. The address and public key are mathematically linked and verifiably consistent.

**Record the sender address here:** ___________________________

-----

## Proof 3: ECDSA Signature Verification

- [ ] “Verify with correct key: VALID” was displayed
- [ ] “Verify with wrong key: INVALID” was displayed
- [ ] Output says: “PROVED: Only the private key holder can sign.”

**What this proves:** ECDSA signatures are unforgeable. Only the person with the private key can produce a valid signature. Anyone else’s attempt fails.

-----

## Proof 4: Block Height Freshness

- [ ] Age 0 blocks: ACCEPTED
- [ ] Age 100 blocks: ACCEPTED
- [ ] Age 1412 blocks: REJECTED
- [ ] Output says: “PROVED: Messages outside the freshness window are rejected.”

**What this proves:** BUTL uses Bitcoin block heights as a timestamp. Messages older than the freshness window (default: 144 blocks, ~24 hours) are rejected, preventing replay attacks.

-----

## Proof 5: Proof of Satoshi (4 Scenarios)

### Scenario A: PoS Disabled

- [ ] Steps 1-4 and 6: all PASS
- [ ] Step 5: DISABLED (skipped)
- [ ] Gate: OPEN
- [ ] Output says: “PROVED: Gate passes without balance check. Pure math only.”

### Scenario B: PoS Enabled, Threshold = 1,000 sat, Balance = 50,000 sat

- [ ] Steps 1-4: all PASS
- [ ] Step 5 (Proof of Satoshi >= 1,000 sat): PASS
- [ ] Step 6: PASS
- [ ] Gate: OPEN
- [ ] Output says: “PROVED: Sender with 50,000 sat passes 1,000 sat threshold.”

### Scenario C: PoS Enabled, Threshold = 100,000 sat, Balance = 50,000 sat

- [ ] Steps 1-4: all PASS
- [ ] Step 5 (Proof of Satoshi >= 100,000 sat): FAIL
- [ ] Gate: CLOSED
- [ ] Output says: “PROVED: 50,000 sat sender REJECTED at 100,000 sat threshold.”

### Scenario D: PoS Enabled, Threshold = 1 sat, Balance = 0 sat

- [ ] Steps 1-4: all PASS
- [ ] Step 5 (Proof of Satoshi >= 1 sat): FAIL
- [ ] Gate: CLOSED
- [ ] Output says: “PROVED: Zero-balance sender REJECTED when PoS enabled.”

**What this proves:** The Proof of Satoshi toggle and slider work correctly. When disabled, the gate operates on pure math. When enabled, it enforces the receiver’s chosen balance threshold. Insufficient funds or zero balance result in rejection.

-----

## Proof 6: Address Chaining

- [ ] Two different Bitcoin addresses were displayed (Message 0 and Message 1)
- [ ] The two addresses are different from each other
- [ ] Chain proof (key_0 signs addr_1): VALID
- [ ] Forged proof (wrong key): INVALID
- [ ] A Thread ID was displayed (64 hex characters)
- [ ] Output says: “PROVED: Fresh address per message, cryptographically linked.”

**What this proves:** Each message uses a new address (forward privacy), but the sender proves it’s still them by signing the new address with the old key (identity continuity). Forging a chain proof without the real key fails.

-----

## Proof 7: Receiver-Only Encryption (ECDH)

- [ ] “Shared secrets match: YES” was displayed
- [ ] The receiver successfully decrypted: “Hello from BUTL v1.2!”
- [ ] Wrong receiver: AUTH FAILED
- [ ] Output says: “PROVED: Only the intended receiver can decrypt.”

**What this proves:** ECDH key agreement produces the same shared secret from both sides (sender and receiver). The message is encrypted with this secret. Only the intended receiver can decrypt it. A different person gets AUTH FAILED.

-----

## Proof 8: Verify-Before-Download Gate (Full 8 Steps)

### Phase 1: Header Only

- [ ] Step 1 (Structural Validation): PASS
- [ ] Step 2 (Receiver Match): PASS
- [ ] Step 3 (Signature / ECDSA): PASS
- [ ] Step 4 (Block Freshness): PASS
- [ ] Step 5 (Proof of Satoshi): DISABLED or PASS
- [ ] Step 6 (Chain Proof): PASS
- [ ] Gate: OPEN

### Phase 2: Payload

- [ ] Step 7 (Payload Hash / SHA-256): PASS
- [ ] Step 8 (Decryption / AEAD): PASS
- [ ] Plaintext displayed: “Hello from BUTL v1.2!”

### Tamper Test

- [ ] Tampered payload: AUTH FAILED (detected)

### Wrong Receiver Test

- [ ] Wrong receiver: Gate CLOSED at Step 2
- [ ] Payload: NEVER DOWNLOADED

**What this proves:** The complete two-phase security model works. Phase 1 verifies the header with no payload on the device. Phase 2 downloads and decrypts the payload only after the gate passes. Tampered payloads are caught. Wrong receivers are blocked before any payload is transmitted.

-----

## Final Summary

- [ ] “ALL 8 PROOFS PASSED” was displayed
- [ ] “Proof of Satoshi: 4 sub-scenarios verified” was displayed
- [ ] “Consistency check: PASS” was displayed
- [ ] “Tampered payload: detected” was displayed
- [ ] “Wrong receiver: rejected” was displayed

-----

## Verification Result

Check one:

- [ ] **ALL PROOFS PASSED.** The BUTL Protocol v1.2 is verified working on this computer. Every claim is proven by the output. No external dependencies. No internet. Pure math.
- [ ] **ONE OR MORE PROOFS FAILED.** (Record which ones below and report as a GitHub issue.)

Failed proofs (if any): ___________________________

Error messages (if any): ___________________________

-----

## Signature

I have personally run the BUTL MVP on the computer described above and verified the results recorded on this checklist.

**Signed:** ___________________________

**Date:** ___________________________

-----

*This checklist records a single verification event. For additional runs (different computers, different dates), print a new copy.*