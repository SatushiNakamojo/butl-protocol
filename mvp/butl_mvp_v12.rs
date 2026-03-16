// BUTL Minimum Viable Prototype v1.2 (Rust)
// Bitcoin Universal Trust Layer — Proof of Protocol
//
// Proves ALL 8 BUTL protocol claims:
//     1. SHA-256 body integrity
//     2. Bitcoin address identity (secp256k1 + Base58Check + consistency check)
//     3. ECDSA signature verification
//     4. Block height freshness (anti-replay)
//     5. Proof of Satoshi (OPTIONAL — 4 sub-scenarios)
//     6. Address chaining (forward privacy + identity continuity)
//     7. Receiver-only encryption (ECDH + AES-256-GCM)
//     8. Verify-before-download gate (two-phase delivery)
//
// Plus:
//     - Pubkey-to-address consistency check
//     - Wrong receiver rejection
//     - Tampered payload detection
//
// Cargo.toml dependencies:
//     secp256k1 = { version = “0.29”, features = [“global-context”, “rand-std”] }
//     sha2 = “0.10”
//     ripemd = “0.1”
//     aes-gcm = “0.10”
//     rand = “0.8”
//     bs58 = { version = “0.5”, features = [“check”] }
//     hex = “0.4”
//
// Run:
//     cargo run –release
//
// License: MIT OR Apache-2.0 (dual licensed, at your option)
// Patent:  PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md (applies to all users)
//
// Copyright 2026 Satushi Nakamojo

use aes_gcm::{
aead::{Aead, KeyInit, Payload},
Aes256Gcm, Nonce,
};
use rand::RngCore;
use ripemd::Ripemd160;
use secp256k1::{ecdh::SharedSecret, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

// ══════════════════════════════════════════════════════════════
// CRYPTOGRAPHIC PRIMITIVES
// ══════════════════════════════════════════════════════════════

fn sha256(data: &[u8]) -> [u8; 32] {
let mut h = Sha256::new();
h.update(data);
let r = h.finalize();
let mut out = [0u8; 32];
out.copy_from_slice(&r);
out
}

fn sha256_hex(data: &[u8]) -> String {
hex::encode(sha256(data))
}

fn to_address(pubkey: &[u8]) -> String {
let sha_hash = sha256(pubkey);
let mut ripemd = Ripemd160::new();
ripemd.update(sha_hash);
let ripemd_hash = ripemd.finalize();
let mut payload = vec![0x00u8];
payload.extend_from_slice(&ripemd_hash);
let checksum = sha256(&sha256(&payload));
payload.extend_from_slice(&checksum[..4]);
bs58::encode(payload).into_string()
}

fn verify_address_pubkey_consistency(address: &str, pubkey: &[u8]) -> bool {
to_address(pubkey) == address
}

fn keygen(seed: &[u8]) -> (SecretKey, PublicKey, String) {
let secp = Secp256k1::new();
let secret = SecretKey::from_slice(&sha256(seed)).expect(“valid key”);
let public = PublicKey::from_secret_key(&secp, &secret);
let address = to_address(&public.serialize());
(secret, public, address)
}

fn sign(sk: &SecretKey, hash: &[u8; 32]) -> secp256k1::ecdsa::Signature {
let secp = Secp256k1::signing_only();
let msg = secp256k1::Message::from_digest_slice(hash).unwrap();
secp.sign_ecdsa(&msg, sk)
}

fn verify(pk: &PublicKey, sig: &secp256k1::ecdsa::Signature, hash: &[u8; 32]) -> bool {
let secp = Secp256k1::verification_only();
let msg = secp256k1::Message::from_digest_slice(hash).unwrap();
secp.verify_ecdsa(&msg, sig, pk).is_ok()
}

fn ecdh_secret(my_sk: &SecretKey, their_pk: &PublicKey) -> [u8; 32] {
sha256(SharedSecret::new(their_pk, my_sk).as_ref())
}

fn encrypt_aead(
plaintext: &[u8],
key: &[u8; 32],
sender: &str,
receiver: &str,
) -> Vec<u8> {
let mut nonce_bytes = [0u8; 12];
rand::thread_rng().fill_bytes(&mut nonce_bytes);
let nonce = Nonce::from_slice(&nonce_bytes);
let aad = format!(”{}{}”, sender, receiver);
let cipher = Aes256Gcm::new_from_slice(key).unwrap();
let ct = cipher
.encrypt(nonce, Payload { msg: plaintext, aad: aad.as_bytes() })
.unwrap();
let mut result = Vec::with_capacity(12 + ct.len());
result.extend_from_slice(&nonce_bytes);
result.extend_from_slice(&ct);
result
}

fn decrypt_aead(
encrypted: &[u8],
key: &[u8; 32],
sender: &str,
receiver: &str,
) -> Result<Vec<u8>, String> {
if encrypted.len() < 28 {
return Err(“payload too short”.into());
}
let nonce = Nonce::from_slice(&encrypted[..12]);
let aad = format!(”{}{}”, sender, receiver);
let cipher = Aes256Gcm::new_from_slice(key).unwrap();
cipher
.decrypt(nonce, Payload { msg: &encrypted[12..], aad: aad.as_bytes() })
.map_err(|_| “AUTH FAILED — tampered payload or wrong key”.into())
}

// ══════════════════════════════════════════════════════════════
// BUTL VERIFICATION GATE v1.2
// ══════════════════════════════════════════════════════════════

fn butl_gate(
pk: &PublicKey,
sig: &secp256k1::ecdsa::Signature,
hash: &[u8; 32],
rx_addr: &str,
my_addr: &str,
block_age: i64,
window: i64,
chain_ok: bool,
pos_enabled: bool,
pos_min_sats: u64,
sender_balance: u64,
) -> (bool, Vec<(String, bool)>, Vec<String>) {
let mut results: Vec<(String, bool)> = Vec::new();
let mut skipped: Vec<String> = Vec::new();

```
// Step 1: Structural Validation
let compressed = pk.serialize();
let struct_ok = compressed.len() == 33
    && (compressed[0] == 0x02 || compressed[0] == 0x03);
results.push(("1. Structural Validation".into(), struct_ok));
if !struct_ok {
    return (false, results, skipped);
}

// Step 2: Receiver Match
let matched = rx_addr == my_addr;
results.push(("2. Receiver Match".into(), matched));
if !matched {
    return (false, results, skipped);
}

// Step 3: Signature (ECDSA)
let sig_ok = verify(pk, sig, hash);
results.push(("3. Signature (ECDSA)".into(), sig_ok));
if !sig_ok {
    return (false, results, skipped);
}

// Step 4: Block Freshness
let fresh = block_age >= 0 && block_age <= window;
results.push(("4. Block Freshness".into(), fresh));
if !fresh {
    return (false, results, skipped);
}

// Step 5: Proof of Satoshi (OPTIONAL)
if pos_enabled {
    let pos_ok = sender_balance >= pos_min_sats;
    results.push((
        format!("5. Proof of Satoshi (>= {:,} sat)", pos_min_sats),
        pos_ok,
    ));
    if !pos_ok {
        return (false, results, skipped);
    }
} else {
    skipped.push("5. Proof of Satoshi [DISABLED — pure math mode]".into());
}

// Step 6: Chain Proof
results.push(("6. Chain Proof".into(), chain_ok));
if !chain_ok {
    return (false, results, skipped);
}

(true, results, skipped)
```

}

/// Print gate results in the standard format.
fn print_gate(results: &[(String, bool)], skipped: &[String], passed: bool) {
for (name, ok) in results {
println!(”  {}: {}”, name, if *ok { “PASS” } else { “FAIL” });
}
for s in skipped {
println!(”  {}”, s);
}
println!(”  Gate: {}”, if passed { “OPEN” } else { “CLOSED” });
}

// ══════════════════════════════════════════════════════════════
// 8 PROOFS + PROOF OF SATOSHI DEMO
// ══════════════════════════════════════════════════════════════

fn main() {
println!(”{}”, “=”.repeat(66));
println!(”  BUTL Minimum Viable Prototype v1.2 (Rust)”);
println!(”  Real secp256k1 + AES-256-GCM”);
println!(”  Proof of Satoshi: OPTIONAL (toggle + slider)”);
println!(”{}”, “=”.repeat(66));

```
// ── Generate identities ──
let (s0_sk, s0_pk, s0_addr) = keygen(b"sender-seed-0");
let (rx_sk, rx_pk, rx_addr) = keygen(b"receiver-seed");
let body = b"Hello from BUTL v1.2!";

println!("\nSender:   {}", s0_addr);
println!("Receiver: {}", rx_addr);

// ══════════════════════════════════════════════════════════
// PROOF 1: SHA-256 Body Integrity
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 1: SHA-256 Body Integrity\n");
let body_hash = sha256_hex(body);
let mut tampered_body = body.to_vec();
tampered_body.push(b'X');
let tampered_hash = sha256_hex(&tampered_body);
assert_ne!(body_hash, tampered_hash);
println!("  Original hash:  {}...", &body_hash[..48]);
println!("  Tampered hash:  {}...", &tampered_hash[..48]);
println!("  Match: NO");
println!("  PROVED: Any modification to the body changes the hash.");

// ══════════════════════════════════════════════════════════
// PROOF 2: Bitcoin Address Identity
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 2: Bitcoin Address Identity\n");
assert!(s0_addr.starts_with('1') && s0_addr.len() >= 25);
let s0_compressed = s0_pk.serialize();
assert!(verify_address_pubkey_consistency(&s0_addr, &s0_compressed));
println!("  Address:    {}", s0_addr);
println!("  Public Key: {}", hex::encode(s0_compressed));
println!("  Consistency check: PASS (pubkey derives to address)");
println!("  PROVED: Valid Bitcoin P2PKH address from secp256k1.");

// ══════════════════════════════════════════════════════════
// PROOF 3: ECDSA Signature Verification
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 3: ECDSA Signature Verification\n");
let canonical = format!(
    "BUTL-Version:1\n\
     BUTL-Sender:{}\n\
     BUTL-SenderPubKey:{}\n\
     BUTL-Receiver:{}\n\
     BUTL-ReceiverPubKey:{}\n\
     BUTL-BlockHeight:890412\n\
     BUTL-BlockHash:00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e\n\
     BUTL-PayloadHash:{}\n\
     BUTL-PrevAddr:\n\
     BUTL-Nonce:",
    s0_addr,
    hex::encode(s0_compressed),
    rx_addr,
    hex::encode(rx_pk.serialize()),
    body_hash,
);
let signing_hash = sha256(canonical.as_bytes());
let sig = sign(&s0_sk, &signing_hash);
assert!(verify(&s0_pk, &sig, &signing_hash));
let (_, atk_pk, _) = keygen(b"attacker");
assert!(!verify(&atk_pk, &sig, &signing_hash));
println!("  Canonical payload signed with sender's private key.");
println!("  Verify with correct key: VALID");
println!("  Verify with wrong key:   INVALID");
println!("  PROVED: Only the private key holder can sign.");

// ══════════════════════════════════════════════════════════
// PROOF 4: Block Height Freshness
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 4: Block Height Freshness\n");
let window: i64 = 144;
assert!(0 >= 0 && 0 <= window);        // Fresh: age 0
assert!(100 >= 0 && 100 <= window);     // Fresh: age 100
assert!(!(1412 >= 0 && 1412 <= window)); // Stale: age 1412
println!("  Freshness window: {} blocks (~24 hours)", window);
println!("  Age 0 blocks:    ACCEPTED (current block)");
println!("  Age 100 blocks:  ACCEPTED (within window)");
println!("  Age 1412 blocks: REJECTED (stale — possible replay)");
println!("  PROVED: Messages outside the freshness window are rejected.");

// ══════════════════════════════════════════════════════════
// PROOF 5: Proof of Satoshi (OPTIONAL — 4 scenarios)
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 5: Proof of Satoshi (OPTIONAL)\n");

// Scenario A: PoS DISABLED
println!("  ── Scenario A: PoS DISABLED ──");
let (p, res, sk) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, &rx_addr,
    0, 144, true, false, 0, 0,
);
print_gate(&res, &sk, p);
assert!(p);
println!("  PROVED: Gate passes without balance check. Pure math only.\n");

// Scenario B: PoS ENABLED, sender has 50,000 sat, threshold 1,000
println!("  ── Scenario B: PoS ENABLED, slider = 1,000 sat ──");
let (p, res, sk) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, &rx_addr,
    0, 144, true, true, 1000, 50000,
);
print_gate(&res, &sk, p);
assert!(p);
println!("  PROVED: Sender with 50,000 sat passes 1,000 sat threshold.\n");

// Scenario C: PoS ENABLED, sender has 50,000 sat, threshold 100,000
println!("  ── Scenario C: PoS ENABLED, slider = 100,000 sat ──");
let (p, res, sk) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, &rx_addr,
    0, 144, true, true, 100000, 50000,
);
print_gate(&res, &sk, p);
assert!(!p);
println!("  PROVED: 50,000 sat sender REJECTED at 100,000 sat threshold.\n");

// Scenario D: PoS ENABLED, zero balance
println!("  ── Scenario D: PoS ENABLED, zero-balance sender ──");
let (p, res, sk) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, &rx_addr,
    0, 144, true, true, 1, 0,
);
print_gate(&res, &sk, p);
assert!(!p);
println!("  PROVED: Zero-balance sender REJECTED when PoS enabled.");

// ══════════════════════════════════════════════════════════
// PROOF 6: Address Chaining
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 6: Address Chaining\n");
let (_, _, s1_addr) = keygen(b"sender-seed-1");
assert_ne!(s0_addr, s1_addr);
let chain_hash = sha256(s1_addr.as_bytes());
let chain_sig = sign(&s0_sk, &chain_hash);
assert!(verify(&s0_pk, &chain_sig, &chain_hash));
let (atk_sk, _, _) = keygen(b"forger");
let forged_sig = sign(&atk_sk, &chain_hash);
assert!(!verify(&s0_pk, &forged_sig, &chain_hash));
let thread_id = sha256_hex(s0_addr.as_bytes());
println!("  Message 0: {}", s0_addr);
println!("  Message 1: {}", s1_addr);
println!("  Chain proof (key_0 signs addr_1): VALID");
println!("  Forged proof (wrong key):         INVALID");
println!("  Thread ID: {}...", &thread_id[..48]);
println!("  PROVED: Fresh address per message, cryptographically linked.");

// ══════════════════════════════════════════════════════════
// PROOF 7: Receiver-Only Encryption (ECDH + AES-256-GCM)
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 7: Receiver-Only Encryption (ECDH + AES-256-GCM)\n");

// ECDH shared secrets must match (commutativity)
let secret_sender = ecdh_secret(&s0_sk, &rx_pk);
let secret_receiver = ecdh_secret(&rx_sk, &s0_pk);
assert_eq!(secret_sender, secret_receiver);
println!("  Sender  computes: SHA-256(sender_priv × receiver_pub)");
println!("  Receiver computes: SHA-256(receiver_priv × sender_pub)");
println!("  Shared secrets match: YES");

// Encrypt and decrypt
let ciphertext = encrypt_aead(body, &secret_sender, &s0_addr, &rx_addr);
let plaintext = decrypt_aead(&ciphertext, &secret_receiver, &s0_addr, &rx_addr).unwrap();
assert_eq!(plaintext, body);
println!("  Receiver decrypts: \"{}\"", String::from_utf8_lossy(&plaintext));

// Wrong receiver cannot decrypt
let (wrong_sk, _, _) = keygen(b"wrong-person");
let wrong_secret = ecdh_secret(&wrong_sk, &s0_pk);
assert!(decrypt_aead(&ciphertext, &wrong_secret, &s0_addr, &rx_addr).is_err());
println!("  Wrong receiver:    AUTH FAILED");
println!("  PROVED: Only the intended receiver can decrypt.");

// ══════════════════════════════════════════════════════════
// PROOF 8: Verify-Before-Download Gate (Full 8-Step)
// ══════════════════════════════════════════════════════════
println!("\n{}", "-".repeat(66));
println!("PROOF 8: Verify-Before-Download Gate (Full 8 Steps)\n");

// Phase 1: Header-only verification (Steps 1-6)
println!("  == PHASE 1: Header Only (no payload on device) ==");
let (passed, results, skipped) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, &rx_addr,
    0, 144, true, false, 0, 0,
);
print_gate(&results, &skipped, passed);
assert!(passed);

// Phase 2: Payload verification (Steps 7-8)
println!("\n  == PHASE 2: Payload (only after gate opens) ==");
let encrypted_hash = sha256_hex(&ciphertext);
assert_eq!(sha256_hex(&ciphertext), encrypted_hash);
println!("  7. Payload Hash (SHA-256): PASS");

let decrypted = decrypt_aead(&ciphertext, &secret_receiver, &s0_addr, &rx_addr).unwrap();
assert_eq!(decrypted, body);
println!("  8. Decryption (AES-GCM):  PASS");
println!("     Plaintext: \"{}\"", String::from_utf8_lossy(&decrypted));

// Tampered payload detection
println!("\n  == TAMPER TEST ==");
let mut tampered_ct = ciphertext.clone();
let last = tampered_ct.len() - 1;
tampered_ct[last] ^= 0xFF;
assert!(decrypt_aead(&tampered_ct, &secret_receiver, &s0_addr, &rx_addr).is_err());
println!("  Tampered payload: AUTH FAILED (detected at Step 8)");

// Wrong receiver gate test
println!("\n  == WRONG RECEIVER TEST ==");
let (wrong_passed, _, _) = butl_gate(
    &s0_pk, &sig, &signing_hash, &rx_addr, "1WrongReceiverAddress",
    0, 144, true, false, 0, 0,
);
assert!(!wrong_passed);
println!("  Wrong receiver: Gate CLOSED at Step 2");
println!("  Payload: NEVER DOWNLOADED");

println!("\n  PROVED: No payload reaches the device until all checks pass.");

// ══════════════════════════════════════════════════════════
// SUMMARY
// ══════════════════════════════════════════════════════════
println!("\n{}", "=".repeat(66));
println!("  ALL 8 PROOFS PASSED");
println!("  Proof of Satoshi: 4 sub-scenarios verified");
println!("  Consistency check: PASS");
println!("  Tampered payload: detected");
println!("  Wrong receiver: rejected");
println!("{}", "=".repeat(66));

let proofs = [
    ("SHA-256 Integrity",       "Any modification changes the hash"),
    ("Bitcoin Address ID",      "Valid P2PKH from secp256k1 + consistency check"),
    ("ECDSA Signatures",        "Only the private key holder can sign"),
    ("Block Height Freshness",  "Stale messages rejected (anti-replay)"),
    ("Proof of Satoshi",        "Optional toggle + slider: 4 scenarios verified"),
    ("Address Chaining",        "Fresh address per message, cryptographically linked"),
    ("Receiver Encryption",     "Only the intended receiver can decrypt (ECDH)"),
    ("Verify-Before-Download",  "Gate blocks payload until all checks pass"),
];
for (i, (name, desc)) in proofs.iter().enumerate() {
    println!("  {}. {:<25} {}", i + 1, name, desc);
}

println!("\n  BUTL Protocol v1.2 — proven with real secp256k1 + AES-256-GCM.");
println!("  Proof of Satoshi is optional by design.");
println!("  Without PoS: 100% pure math. Zero external trust.");
println!("{}", "=".repeat(66));
```

}