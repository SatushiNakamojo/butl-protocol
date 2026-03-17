//! BUTL Reference Implementation v1.2 (Rust)
//! Bitcoin Universal Trust Layer
//!
//! Complete, production-ready API for building BUTL applications.
//!
//! Implements all protocol features from the v1.2 specification:
//!   - Self-sovereign identity (Bitcoin addresses from secp256k1)
//!   - End-to-end encryption (ECDH + AES-256-GCM)
//!   - Message integrity (SHA-256)
//!   - Replay protection (block height freshness)
//!   - Address chaining (new key per message, cryptographically linked)
//!   - Verify-before-download gate (8-step, two-phase)
//!   - Proof of Satoshi (optional, toggle + slider)
//!   - Pubkey-to-address consistency check
//!
//! # Cargo Dependencies
//!
//! ```toml
//! [package]
//! name = "butl"
//! version = "1.2.0"
//! edition = "2021"
//! description = "Bitcoin Universal Trust Layer - reference implementation"
//! license = "MIT OR Apache-2.0"
//!
//! [dependencies]
//! secp256k1 = { version = "0.29", features = ["global-context", "rand-std"] }
//! sha2 = "0.10"
//! ripemd = "0.1"
//! aes-gcm = "0.10"
//! base64 = "0.22"
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! bs58 = { version = "0.5", features = ["check"] }
//! hex = "0.4"
//! ```
//!
//! # Quick Start
//!
//! ```bash
//! cargo new butl-demo && cd butl-demo
//! # Copy Cargo.toml dependencies from above
//! # Copy this file to src/main.rs
//! cargo run --release
//! ```
//!
//! License: MIT OR Apache-2.0 (dual licensed, at your option)
//! Patent:  PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md (applies to all users)
//!
//! Copyright 2026 Satushi Nakamojo

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use ripemd::Ripemd160;
use secp256k1::{ecdh::SharedSecret, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Library version.
pub const VERSION: &str = "1.2.0";

/// BUTL protocol version (the wire format version).
pub const PROTOCOL_VERSION: u32 = 1;


// ==============================================================
// ERROR TYPE
// ==============================================================

/// Error type for all BUTL operations.
#[derive(Debug, Clone)]
pub enum BUTLError {
    Encryption(String),
    Decryption(String),
    Signing(String),
    Verification(String),
    InvalidKey(String),
    ConsistencyCheck(String),
}

impl fmt::Display for BUTLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BUTLError::Encryption(s) => write!(f, "Encryption: {}", s),
            BUTLError::Decryption(s) => write!(f, "Decryption: {}", s),
            BUTLError::Signing(s) => write!(f, "Signing: {}", s),
            BUTLError::Verification(s) => write!(f, "Verification: {}", s),
            BUTLError::InvalidKey(s) => write!(f, "InvalidKey: {}", s),
            BUTLError::ConsistencyCheck(s) => write!(f, "ConsistencyCheck: {}", s),
        }
    }
}

impl std::error::Error for BUTLError {}


// ==============================================================
// CRYPTOGRAPHIC PRIMITIVES
// ==============================================================

/// SHA-256 hash. Returns 32 bytes.
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    let r = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}

/// SHA-256 hash. Returns 64 lowercase hex characters.
fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

/// Bitcoin-style double SHA-256: SHA-256(SHA-256(data)).
fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}


// -- Base58Check ---------------------------------------------

/// Compressed secp256k1 public key (33 bytes) -> Bitcoin P2PKH address.
fn to_address(pubkey: &[u8]) -> String {
    let sha_hash = sha256(pubkey);
    let mut ripemd = Ripemd160::new();
    ripemd.update(sha_hash);
    let ripemd_hash = ripemd.finalize();
    let mut payload = vec![0x00u8];
    payload.extend_from_slice(&ripemd_hash);
    let checksum = double_sha256(&payload);
    payload.extend_from_slice(&checksum[..4]);
    bs58::encode(payload).into_string()
}

/// Verify that a compressed public key (hex) correctly derives to the
/// claimed Bitcoin address. Returns true if consistent, false if mismatched.
///
/// This is the consistency check from BUTL Protocol Specification S5.4.
/// Receivers MUST verify this at Step 1 (Structural Validation).
fn verify_address_pubkey_consistency(address: &str, pubkey_hex: &str) -> bool {
    match hex::decode(pubkey_hex) {
        Ok(pk_bytes) => to_address(&pk_bytes) == address,
        Err(_) => false,
    }
}


// -- Encryption / Decryption ---------------------------------

/// AES-256-GCM encrypt.
/// Returns: IV (12 bytes) || ciphertext || GCM tag (16 bytes).
/// AAD = UTF-8(sender_address || receiver_address).
fn encrypt_payload(
    plaintext: &[u8],
    key: &[u8; 32],
    sender: &str,
    receiver: &str,
) -> Result<Vec<u8>, BUTLError> {
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);
    let aad = format!("{}{}", sender, receiver);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| BUTLError::Encryption(format!("{}", e)))?;
    let ct = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad: aad.as_bytes() })
        .map_err(|e| BUTLError::Encryption(format!("{}", e)))?;
    let mut result = Vec::with_capacity(12 + ct.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ct);
    Ok(result)
}

/// AES-256-GCM decrypt.
/// Input: IV (12 bytes) || ciphertext || GCM tag (16 bytes).
/// Returns decrypted plaintext or an error on authentication failure.
fn decrypt_payload(
    encrypted: &[u8],
    key: &[u8; 32],
    sender: &str,
    receiver: &str,
) -> Result<Vec<u8>, BUTLError> {
    if encrypted.len() < 28 {
        return Err(BUTLError::Decryption("payload too short".into()));
    }
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let aad = format!("{}{}", sender, receiver);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| BUTLError::Decryption(format!("{}", e)))?;
    cipher
        .decrypt(nonce, Payload { msg: &encrypted[12..], aad: aad.as_bytes() })
        .map_err(|_| BUTLError::Decryption(
            "GCM auth failed - tampered or wrong key".into()
        ))
}


// ==============================================================
// BUTL KEYPAIR
// ==============================================================

/// A single secp256k1 keypair for BUTL.
///
/// Provides Bitcoin address derivation, ECDSA signing/verification,
/// and ECDH shared secret computation.
///
/// # Examples
///
/// ```
/// // Generate a random identity
/// let me = BUTLKeypair::generate().unwrap();
///
/// // From a specific 32-byte seed (deterministic)
/// let me = BUTLKeypair::from_secret(&sha256(b"my-seed")).unwrap();
///
/// // Access identity components
/// println!("Address:    {}", me.address);
/// println!("Public Key: {}", me.public_key_hex);
/// ```
#[derive(Clone)]
pub struct BUTLKeypair {
    secret_key: SecretKey,
    public_key: PublicKey,
    /// Bitcoin P2PKH address (Base58Check encoded).
    pub address: String,
    /// Compressed secp256k1 public key (66 hex characters).
    pub public_key_hex: String,
}

impl BUTLKeypair {
    /// Create a keypair from a 32-byte private key (deterministic).
    pub fn from_secret(bytes: &[u8; 32]) -> Result<Self, BUTLError> {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(bytes)
            .map_err(|e| BUTLError::InvalidKey(format!("{}", e)))?;
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let compressed = pk.serialize();
        Ok(Self {
            secret_key: sk,
            public_key: pk,
            address: to_address(&compressed),
            public_key_hex: hex::encode(compressed),
        })
    }

    /// Generate a random keypair.
    pub fn generate() -> Result<Self, BUTLError> {
        let secp = Secp256k1::new();
        let (sk, pk) = secp.generate_keypair(&mut rand::thread_rng());
        let compressed = pk.serialize();
        Ok(Self {
            secret_key: sk,
            public_key: pk,
            address: to_address(&compressed),
            public_key_hex: hex::encode(compressed),
        })
    }

    /// Compressed public key as 33 bytes.
    pub fn public_key_bytes(&self) -> [u8; 33] {
        self.public_key.serialize()
    }

    /// Private key as 32 bytes.
    /// NEVER log, transmit, or store this unencrypted.
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.secret_key.secret_bytes()
    }

    /// ECDSA sign a 32-byte SHA-256 hash. Returns DER-encoded signature.
    pub fn sign(&self, hash: &[u8; 32]) -> Result<Vec<u8>, BUTLError> {
        let secp = Secp256k1::signing_only();
        let msg = secp256k1::Message::from_digest_slice(hash)
            .map_err(|e| BUTLError::Signing(format!("{}", e)))?;
        Ok(secp.sign_ecdsa(&msg, &self.secret_key).serialize_der().to_vec())
    }

    /// Verify an ECDSA signature against a compressed public key.
    /// Returns true if valid, false if invalid.
    pub fn verify(pubkey: &[u8], sig: &[u8], hash: &[u8; 32]) -> Result<bool, BUTLError> {
        let secp = Secp256k1::verification_only();
        let pk = PublicKey::from_slice(pubkey)
            .map_err(|e| BUTLError::Verification(format!("pubkey: {}", e)))?;
        let msg = secp256k1::Message::from_digest_slice(hash)
            .map_err(|e| BUTLError::Verification(format!("msg: {}", e)))?;
        let s = secp256k1::ecdsa::Signature::from_der(sig)
            .map_err(|e| BUTLError::Verification(format!("sig: {}", e)))?;
        Ok(secp.verify_ecdsa(&msg, &s, &pk).is_ok())
    }

    /// Compute ECDH shared secret: SHA-256((my_priv x their_pub).x).
    /// `other` is a 33-byte compressed secp256k1 public key.
    pub fn ecdh(&self, other: &[u8]) -> Result<[u8; 32], BUTLError> {
        let pk = PublicKey::from_slice(other)
            .map_err(|e| BUTLError::InvalidKey(format!("{}", e)))?;
        Ok(sha256(SharedSecret::new(&pk, &self.secret_key).as_ref()))
    }
}


// ==============================================================
// BUTL KEYCHAIN
// ==============================================================

/// Deterministic key derivation with automatic address chaining.
///
/// Derives a sequence of keypairs from a master seed. Each call to
/// `next_keypair()` advances the chain: the current keypair becomes
/// the previous (retained for chain proof), and a new keypair is
/// generated.
///
/// # Examples
///
/// ```
/// let mut keychain = BUTLKeychain::new(sha256(b"my-seed"));
/// let kp0 = keychain.next_keypair().unwrap();  // genesis
/// let kp1 = keychain.next_keypair().unwrap();  // chained
/// let proof = keychain.create_chain_proof(&kp1.address).unwrap();
/// ```
pub struct BUTLKeychain {
    seed: [u8; 32],
    index: u32,
    current: Option<BUTLKeypair>,
    previous: Option<BUTLKeypair>,
    history: Vec<String>,
}

impl BUTLKeychain {
    /// Create a new keychain from a 32-byte master seed.
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            seed,
            index: 0,
            current: None,
            previous: None,
            history: Vec::new(),
        }
    }

    /// Derive a private key from seed + index.
    /// Simplified BIP-32 style. Production: use full BIP-32 HD derivation.
    fn derive_key(&self, idx: u32) -> [u8; 32] {
        let mut data = Vec::with_capacity(36);
        data.extend_from_slice(&self.seed);
        data.extend_from_slice(&idx.to_be_bytes());
        sha256(&data)
    }

    /// Generate the next keypair in the chain.
    /// The previous keypair is retained for chain proof creation.
    pub fn next_keypair(&mut self) -> Result<&BUTLKeypair, BUTLError> {
        self.previous = self.current.take();
        let kp = BUTLKeypair::from_secret(&self.derive_key(self.index))?;
        self.history.push(kp.address.clone());
        self.current = Some(kp);
        self.index += 1;
        Ok(self.current.as_ref().unwrap())
    }

    /// The current (most recent) keypair.
    pub fn current(&self) -> Option<&BUTLKeypair> {
        self.current.as_ref()
    }

    /// The previous keypair (for chain proof creation).
    pub fn previous(&self) -> Option<&BUTLKeypair> {
        self.previous.as_ref()
    }

    /// The previous keypair's address (for BUTL-PrevAddr).
    /// Returns empty string for genesis (no previous).
    pub fn previous_address(&self) -> String {
        self.previous
            .as_ref()
            .map(|k| k.address.clone())
            .unwrap_or_default()
    }

    /// Sign the current address with the previous key (chain proof).
    /// Returns empty Vec for genesis (no previous key).
    pub fn create_chain_proof(&self, addr: &str) -> Result<Vec<u8>, BUTLError> {
        match &self.previous {
            None => Ok(Vec::new()),
            Some(prev) => prev.sign(&sha256(addr.as_bytes())),
        }
    }

    /// Thread ID = SHA-256_HEX(first address in chain).
    /// Constant across all messages in the thread.
    pub fn thread_id(&self) -> String {
        if self.history.is_empty() {
            String::new()
        } else {
            sha256_hex(self.history[0].as_bytes())
        }
    }

    /// Current sequence number (0-indexed).
    pub fn seq_num(&self) -> u32 {
        if self.index == 0 { 0 } else { self.index - 1 }
    }

    /// All addresses generated by this keychain.
    pub fn history(&self) -> &[String] {
        &self.history
    }
}


// ==============================================================
// BUTL HEADER
// ==============================================================

/// BUTL protocol header v1.2.
///
/// Contains all metadata required for the verification gate.
/// Transmitted in cleartext during Phase 1 (header-only delivery).
///
/// See HEADER_REGISTRY.md for the complete field specification.
/// See BUTL_Protocol_Specification_v1.2.md S7 for the canonical
/// signing payload format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BUTLHeader {
    // -- Required fields --
    pub version: u32,
    pub sender: String,
    pub sender_pubkey: String,
    pub receiver: String,
    pub receiver_pubkey: String,
    pub block_height: u64,
    pub block_hash: String,
    pub payload_hash: String,
    pub signature: String,
    pub prev_addr: String,
    pub chain_proof: String,
    pub enc_algo: String,

    // -- Optional fields --
    pub ephemeral_pubkey: String,
    pub nonce: String,
    pub timestamp: u64,
    pub thread_id: String,
    pub seq_num: u32,
    pub payload_size: usize,
}

impl BUTLHeader {
    /// Create a new header with default values.
    pub fn new() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            enc_algo: "AES-256-GCM".into(),
            ..Default::default()
        }
    }

    /// Build the deterministic canonical signing payload.
    /// Fields in fixed order, separated by newline (0x0A).
    /// Empty strings for absent optional values.
    pub fn canonical_payload(&self) -> String {
        format!(
            "BUTL-Version:{}\n\
             BUTL-Sender:{}\n\
             BUTL-SenderPubKey:{}\n\
             BUTL-Receiver:{}\n\
             BUTL-ReceiverPubKey:{}\n\
             BUTL-BlockHeight:{}\n\
             BUTL-BlockHash:{}\n\
             BUTL-PayloadHash:{}\n\
             BUTL-PrevAddr:{}\n\
             BUTL-Nonce:{}",
            self.version, self.sender, self.sender_pubkey,
            self.receiver, self.receiver_pubkey,
            self.block_height, self.block_hash,
            self.payload_hash, self.prev_addr, self.nonce,
        )
    }

    /// Serialize to text header format (email X-headers / HTTP headers).
    /// Format: 'Key: Value' with one space after colon.
    pub fn to_text_headers(&self) -> String {
        let mut lines = vec![
            format!("BUTL-Version: {}", self.version),
            format!("BUTL-Sender: {}", self.sender),
            format!("BUTL-SenderPubKey: {}", self.sender_pubkey),
            format!("BUTL-Receiver: {}", self.receiver),
            format!("BUTL-ReceiverPubKey: {}", self.receiver_pubkey),
            format!("BUTL-BlockHeight: {}", self.block_height),
            format!("BUTL-BlockHash: {}", self.block_hash),
            format!("BUTL-PayloadHash: {}", self.payload_hash),
            format!("BUTL-Signature: {}", self.signature),
            format!("BUTL-PrevAddr: {}", self.prev_addr),
            format!("BUTL-ChainProof: {}", self.chain_proof),
            format!("BUTL-EncAlgo: {}", self.enc_algo),
        ];
        if !self.ephemeral_pubkey.is_empty() {
            lines.push(format!("BUTL-EphemeralPubKey: {}", self.ephemeral_pubkey));
        }
        if !self.nonce.is_empty() {
            lines.push(format!("BUTL-Nonce: {}", self.nonce));
        }
        if self.timestamp > 0 {
            lines.push(format!("BUTL-Timestamp: {}", self.timestamp));
        }
        if !self.thread_id.is_empty() {
            lines.push(format!("BUTL-ThreadID: {}", self.thread_id));
        }
        if self.seq_num > 0 {
            lines.push(format!("BUTL-SeqNum: {}", self.seq_num));
        }
        if self.payload_size > 0 {
            lines.push(format!("BUTL-PayloadSize: {}", self.payload_size));
        }
        lines.join("\n")
    }

    /// Serialize to pretty-printed JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}


// ==============================================================
// BLOCKCHAIN INTERFACE
// ==============================================================

/// Abstract interface for Bitcoin blockchain queries.
///
/// The reference implementation includes [`StubBlockchain`] that returns
/// hardcoded values. Production deployments MUST replace this with a
/// real implementation connecting to Bitcoin Core RPC, Electrum server,
/// or a REST API (e.g., mempool.space).
///
/// See SECURITY.md for known limitations of the stub.
pub trait BlockchainProvider {
    /// Returns (block_height, block_hash) of the current chain tip.
    fn get_current_block(&self) -> (u64, String);
    /// Verify that block_hash matches the block at the given height.
    fn verify_block(&self, height: u64, hash: &str) -> bool;
    /// Returns the current chain tip height.
    fn get_chain_height(&self) -> u64;
    /// Check if address holds at least min_sats satoshis.
    /// Only called when Proof of Satoshi is enabled.
    fn check_balance(&self, address: &str, min_sats: u64) -> bool;
}

/// Stub blockchain provider for development and testing.
/// Returns hardcoded values. NOT for production use.
pub struct StubBlockchain;

impl BlockchainProvider for StubBlockchain {
    fn get_current_block(&self) -> (u64, String) {
        (
            890412,
            "00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e".into(),
        )
    }
    fn verify_block(&self, _h: u64, _bh: &str) -> bool { true }
    fn get_chain_height(&self) -> u64 { 890412 }
    fn check_balance(&self, _a: &str, _m: u64) -> bool { true }
}


// ==============================================================
// BUTL MESSAGE
// ==============================================================

/// A complete BUTL message: header + encrypted payload.
/// The header is delivered in Phase 1. The payload in Phase 2.
pub struct BUTLMessage {
    pub header: BUTLHeader,
    pub encrypted_payload: Vec<u8>,
}


// ==============================================================
// PROOF OF SATOSHI CONFIGURATION
// ==============================================================

/// Receiver-side Proof of Satoshi configuration.
///
/// When `enabled` is false (default), Step 5 is skipped entirely
/// and BUTL operates as pure math with zero external dependencies.
pub struct ProofOfSatoshiConfig {
    /// Whether to enforce the balance check (Step 5).
    pub enabled: bool,
    /// Minimum balance required. Range: 1 - 100,000,000 (1 BTC).
    pub min_satoshis: u64,
}

impl Default for ProofOfSatoshiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_satoshis: 1,
        }
    }
}


// ==============================================================
// SIGNER (SENDER SIDE)
// ==============================================================

/// Signs and encrypts messages with the BUTL protocol.
///
/// Manages the sender's keychain (fresh address per message),
/// constructs the BUTL header, signs the canonical payload, encrypts
/// the message body, and produces a complete [`BUTLMessage`].
///
/// # Examples
///
/// ```
/// let keychain = BUTLKeychain::new(sha256(b"sender-seed"));
/// let mut signer = BUTLSigner::new(keychain, StubBlockchain);
/// let msg = signer.sign_and_encrypt(
///     b"Hello!",
///     &receiver.public_key_bytes(),
///     &receiver.address,
/// ).unwrap();
/// ```
pub struct BUTLSigner<B: BlockchainProvider> {
    pub keychain: BUTLKeychain,
    blockchain: B,
}

impl<B: BlockchainProvider> BUTLSigner<B> {
    /// Create a new signer with the given keychain and blockchain provider.
    pub fn new(keychain: BUTLKeychain, blockchain: B) -> Self {
        Self { keychain, blockchain }
    }

    /// Sign and encrypt a message for a specific receiver.
    ///
    /// Returns a [`BUTLMessage`] with header and encrypted payload,
    /// ready for two-phase transmission.
    pub fn sign_and_encrypt(
        &mut self,
        body: &[u8],
        rx_pubkey: &[u8],
        rx_addr: &str,
    ) -> Result<BUTLMessage, BUTLError> {
        // 1. Fresh sender keypair (address chaining)
        let kp = self.keychain.next_keypair()?.clone();

        // 2. ECDH key agreement
        let shared = kp.ecdh(rx_pubkey)?;

        // 3. Encrypt payload
        let encrypted = encrypt_payload(body, &shared, &kp.address, rx_addr)?;

        // 4. Hash encrypted payload
        let ph = sha256_hex(&encrypted);

        // 5. Current block reference
        let (bh, bhash) = self.blockchain.get_current_block();

        // 6. Random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // 7. Timestamp
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 8. Build header
        let mut header = BUTLHeader {
            version: PROTOCOL_VERSION,
            sender: kp.address.clone(),
            sender_pubkey: kp.public_key_hex.clone(),
            receiver: rx_addr.into(),
            receiver_pubkey: hex::encode(rx_pubkey),
            block_height: bh,
            block_hash: bhash,
            payload_hash: ph,
            prev_addr: self.keychain.previous_address(),
            enc_algo: "AES-256-GCM".into(),
            nonce: hex::encode(nonce_bytes),
            timestamp: ts,
            thread_id: self.keychain.thread_id(),
            seq_num: self.keychain.seq_num(),
            payload_size: encrypted.len(),
            ..Default::default()
        };

        // 9. Chain proof
        if self.keychain.previous().is_some() {
            let proof = self.keychain.create_chain_proof(&kp.address)?;
            header.chain_proof = BASE64.encode(&proof);
        }

        // 10. Sign canonical payload
        let sig = kp.sign(&sha256(header.canonical_payload().as_bytes()))?;
        header.signature = BASE64.encode(&sig);

        Ok(BUTLMessage {
            header,
            encrypted_payload: encrypted,
        })
    }
}


// ==============================================================
// GATE REPORT
// ==============================================================

/// Result of a single verification step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckResult {
    Pass,
    Fail,
    Skip,
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckResult::Pass => write!(f, "PASS"),
            CheckResult::Fail => write!(f, "FAIL"),
            CheckResult::Skip => write!(f, "SKIP"),
        }
    }
}

/// Detailed report of all 8 verification steps.
///
/// Use `gate_passed()` to check if pre-download checks (steps 1-6) passed.
/// Use `fully_verified()` to check if all 8 steps passed.
/// Use `summary()` for a human-readable report.
#[derive(Debug, Clone)]
pub struct GateReport {
    pub structural: CheckResult,
    pub receiver_match: CheckResult,
    pub signature: CheckResult,
    pub freshness: CheckResult,
    pub pos: CheckResult,
    pub chain: CheckResult,
    pub payload_hash: CheckResult,
    pub decryption: CheckResult,
    pub errors: Vec<String>,
}

impl GateReport {
    fn new() -> Self {
        Self {
            structural: CheckResult::Skip,
            receiver_match: CheckResult::Skip,
            signature: CheckResult::Skip,
            freshness: CheckResult::Skip,
            pos: CheckResult::Skip,
            chain: CheckResult::Skip,
            payload_hash: CheckResult::Skip,
            decryption: CheckResult::Skip,
            errors: Vec::new(),
        }
    }

    /// True if all enabled pre-download gate checks (steps 1-6) passed.
    pub fn gate_passed(&self) -> bool {
        let mut checks = vec![
            self.structural, self.receiver_match,
            self.signature, self.freshness,
        ];
        if self.pos != CheckResult::Skip {
            checks.push(self.pos);
        }
        if self.chain != CheckResult::Skip {
            checks.push(self.chain);
        }
        checks.iter().all(|c| *c == CheckResult::Pass)
    }

    /// True if ALL 8 checks passed, including payload verification.
    pub fn fully_verified(&self) -> bool {
        self.gate_passed()
            && self.payload_hash == CheckResult::Pass
            && self.decryption == CheckResult::Pass
    }

    /// Human-readable verification report.
    pub fn summary(&self) -> String {
        let status = if self.fully_verified() {
            "FULLY VERIFIED"
        } else if self.gate_passed() {
            "GATE PASSED"
        } else {
            "REJECTED AT GATE"
        };
        let gate = if self.gate_passed() { "OPEN" } else { "CLOSED" };
        let mut lines = vec![
            format!("BUTL Verification: {}", status),
            String::new(),
            "  == BUTL GATE (header-only, pre-download) ==".into(),
            format!("  1. Structural Validation  : {}", self.structural),
            format!("  2. Receiver Match         : {}", self.receiver_match),
            format!("  3. Signature (ECDSA)      : {}", self.signature),
            format!("  4. Block Freshness        : {}", self.freshness),
            format!("  5. Proof of Satoshi       : {}", self.pos),
            format!("  6. Chain Proof            : {}", self.chain),
            format!("  Gate: {}", gate),
            String::new(),
            "  == POST-GATE (payload verification) ==".into(),
            format!("  7. Payload Hash (SHA-256) : {}", self.payload_hash),
            format!("  8. Decryption (AES-GCM)   : {}", self.decryption),
        ];
        if !self.errors.is_empty() {
            lines.push(String::new());
            lines.push("  Errors:".into());
            for e in &self.errors {
                lines.push(format!("    - {}", e));
            }
        }
        lines.join("\n")
    }
}


// ==============================================================
// BUTL GATE (RECEIVER SIDE)
// ==============================================================

/// Verify-Before-Download Gate v1.2.
///
/// Enforces the BUTL security model: no payload data touches the
/// device until all header checks pass.
///
/// # Two-Phase Usage
///
/// ```
/// let gate = BUTLGate::new(my_keypair, StubBlockchain, 144, Default::default());
///
/// // Phase 1: Header only (no payload on device)
/// let mut report = gate.check_header(&header, None);
///
/// if report.gate_passed() {
///     // Phase 2: Download and decrypt
///     if let Some(plaintext) = gate.accept_payload(&header, &payload, &mut report) {
///         println!("Verified: {}", String::from_utf8_lossy(&plaintext));
///     }
/// }
/// ```
pub struct BUTLGate<B: BlockchainProvider> {
    receiver: BUTLKeypair,
    blockchain: B,
    freshness_window: u64,
    pos_config: ProofOfSatoshiConfig,
}

impl<B: BlockchainProvider> BUTLGate<B> {
    /// Create a new gate with the given receiver keypair, blockchain provider,
    /// freshness window, and Proof of Satoshi configuration.
    pub fn new(
        rx: BUTLKeypair,
        bc: B,
        window: u64,
        pos: ProofOfSatoshiConfig,
    ) -> Self {
        Self {
            receiver: rx,
            blockchain: bc,
            freshness_window: window,
            pos_config: pos,
        }
    }

    /// Phase 1: Header-only verification (Steps 1-6).
    /// No payload data is involved.
    ///
    /// `prev_pk`: The previous sender's compressed public key (33 bytes),
    /// for chain proof verification in Step 6. None for genesis messages.
    pub fn check_header(
        &self,
        h: &BUTLHeader,
        prev_pk: Option<&[u8]>,
    ) -> GateReport {
        let mut r = GateReport::new();

        // -- Step 1: Structural Validation --
        if h.version != PROTOCOL_VERSION {
            r.errors.push(format!(
                "Unsupported version: {} (supports {})",
                h.version, PROTOCOL_VERSION
            ));
            r.structural = CheckResult::Fail;
            return r;
        }

        let checks: Vec<(&str, &str, Option<usize>)> = vec![
            ("Sender", &h.sender, None),
            ("SenderPubKey", &h.sender_pubkey, Some(66)),
            ("Receiver", &h.receiver, None),
            ("ReceiverPubKey", &h.receiver_pubkey, Some(66)),
            ("BlockHash", &h.block_hash, Some(64)),
            ("PayloadHash", &h.payload_hash, Some(64)),
            ("Signature", &h.signature, None),
        ];

        for (name, value, expected_len) in &checks {
            if value.is_empty() {
                r.errors.push(format!("Missing: BUTL-{}", name));
                r.structural = CheckResult::Fail;
                return r;
            }
            if let Some(len) = expected_len {
                if value.len() != *len {
                    r.errors.push(format!(
                        "BUTL-{}: expected {} chars, got {}",
                        name, len, value.len()
                    ));
                    r.structural = CheckResult::Fail;
                    return r;
                }
            }
        }

        // Consistency check: pubkey must derive to address
        if !verify_address_pubkey_consistency(&h.sender, &h.sender_pubkey) {
            r.errors.push(
                "Consistency check failed: BUTL-SenderPubKey does not derive to BUTL-Sender"
                    .into(),
            );
            r.structural = CheckResult::Fail;
            return r;
        }

        r.structural = CheckResult::Pass;

        // -- Step 2: Receiver Match --
        if h.receiver != self.receiver.address {
            r.errors.push(format!(
                "Not for this receiver: {} != {}",
                h.receiver, self.receiver.address
            ));
            r.receiver_match = CheckResult::Fail;
            return r;
        }
        r.receiver_match = CheckResult::Pass;

        // -- Step 3: Signature Verification --
        let canonical_hash = sha256(h.canonical_payload().as_bytes());
        let sig = match BASE64.decode(&h.signature) {
            Ok(b) => b,
            Err(e) => {
                r.errors.push(format!("Bad signature encoding: {}", e));
                r.signature = CheckResult::Fail;
                return r;
            }
        };
        let spk = match hex::decode(&h.sender_pubkey) {
            Ok(b) => b,
            Err(e) => {
                r.errors.push(format!("Bad pubkey hex: {}", e));
                r.signature = CheckResult::Fail;
                return r;
            }
        };
        match BUTLKeypair::verify(&spk, &sig, &canonical_hash) {
            Ok(true) => r.signature = CheckResult::Pass,
            Ok(false) => {
                r.errors.push("ECDSA signature verification failed".into());
                r.signature = CheckResult::Fail;
                return r;
            }
            Err(e) => {
                r.errors.push(format!("Signature error: {}", e));
                r.signature = CheckResult::Fail;
                return r;
            }
        }

        // -- Step 4: Block Freshness --
        if !self.blockchain.verify_block(h.block_height, &h.block_hash) {
            r.errors.push("Block hash does not match claimed height".into());
            r.freshness = CheckResult::Fail;
            return r;
        }
        let tip = self.blockchain.get_chain_height();
        if h.block_height > tip {
            r.errors.push("Block height ahead of chain tip".into());
            r.freshness = CheckResult::Fail;
            return r;
        }
        if tip - h.block_height > self.freshness_window {
            r.errors.push(format!(
                "Block age {} exceeds window {}",
                tip - h.block_height,
                self.freshness_window
            ));
            r.freshness = CheckResult::Fail;
            return r;
        }
        r.freshness = CheckResult::Pass;

        // -- Step 5: Proof of Satoshi (OPTIONAL) --
        if self.pos_config.enabled {
            if !self
                .blockchain
                .check_balance(&h.sender, self.pos_config.min_satoshis)
            {
                r.errors.push(format!(
                    "Proof of Satoshi: {} holds < {} sat",
                    h.sender, self.pos_config.min_satoshis
                ));
                r.pos = CheckResult::Fail;
                return r;
            }
            r.pos = CheckResult::Pass;
        } else {
            r.pos = CheckResult::Skip;
        }

        // -- Step 6: Chain Proof --
        if !h.prev_addr.is_empty() {
            if h.chain_proof.is_empty() {
                r.errors
                    .push("Threaded message but no ChainProof".into());
                r.chain = CheckResult::Fail;
                return r;
            }
            let ch = sha256(h.sender.as_bytes());
            let cp = match BASE64.decode(&h.chain_proof) {
                Ok(b) => b,
                Err(e) => {
                    r.errors.push(format!("Bad chain proof encoding: {}", e));
                    r.chain = CheckResult::Fail;
                    return r;
                }
            };
            match prev_pk {
                Some(pk) => match BUTLKeypair::verify(pk, &cp, &ch) {
                    Ok(true) => r.chain = CheckResult::Pass,
                    Ok(false) => {
                        r.errors.push("Chain proof verification failed".into());
                        r.chain = CheckResult::Fail;
                        return r;
                    }
                    Err(e) => {
                        r.errors.push(format!("Chain proof error: {}", e));
                        r.chain = CheckResult::Fail;
                        return r;
                    }
                },
                None => {
                    r.errors.push("No previous pubkey for chain verification".into());
                    r.chain = CheckResult::Fail;
                    return r;
                }
            }
        } else {
            r.chain = CheckResult::Pass;
        }

        r
    }

    /// Phase 2: Download and decrypt the payload.
    /// Only call if `report.gate_passed()` is true.
    ///
    /// Returns `Some(plaintext)` on success, `None` on failure.
    /// The report is updated with Step 7 and Step 8 results.
    pub fn accept_payload(
        &self,
        h: &BUTLHeader,
        enc: &[u8],
        r: &mut GateReport,
    ) -> Option<Vec<u8>> {
        if !r.gate_passed() {
            r.errors.push("GATE CLOSED - payload rejected".into());
            return None;
        }

        // -- Step 7: Payload Hash --
        if sha256_hex(enc) != h.payload_hash {
            r.errors
                .push("Payload hash mismatch - tampered in transit".into());
            r.payload_hash = CheckResult::Fail;
            return None;
        }
        r.payload_hash = CheckResult::Pass;

        // -- Step 8: Decryption --
        let pk_hex = if !h.ephemeral_pubkey.is_empty() {
            &h.ephemeral_pubkey
        } else {
            &h.sender_pubkey
        };
        let pk = match hex::decode(pk_hex) {
            Ok(b) => b,
            Err(e) => {
                r.errors.push(format!("Bad ECDH pubkey: {}", e));
                r.decryption = CheckResult::Fail;
                return None;
            }
        };
        let ss = match self.receiver.ecdh(&pk) {
            Ok(s) => s,
            Err(e) => {
                r.errors.push(format!("ECDH failed: {}", e));
                r.decryption = CheckResult::Fail;
                return None;
            }
        };
        match decrypt_payload(enc, &ss, &h.sender, &h.receiver) {
            Ok(pt) => {
                r.decryption = CheckResult::Pass;
                Some(pt)
            }
            Err(e) => {
                r.errors.push(format!("{}", e));
                r.decryption = CheckResult::Fail;
                None
            }
        }
    }
}


// ==============================================================
// DEMO
// ==============================================================

fn main() {
    println!("{}", "=".repeat(66));
    println!("  BUTL Reference Implementation v1.2 (Rust)");
    println!("  Protocol version: {}", PROTOCOL_VERSION);
    println!("  Library version:  {}", VERSION);
    println!("{}", "=".repeat(66));

    // -- Setup --
    let sender_kc = BUTLKeychain::new(sha256(b"demo-sender-seed"));
    let rx = BUTLKeypair::from_secret(&sha256(b"demo-receiver-seed")).unwrap();
    let mut signer = BUTLSigner::new(sender_kc, StubBlockchain);
    let gate = BUTLGate::new(
        rx.clone(),
        StubBlockchain,
        144,
        ProofOfSatoshiConfig::default(),
    );

    println!("\nReceiver: {}", rx.address);

    // -- Message 1: Genesis --
    println!("\n{}\nMESSAGE 1 (Genesis)\n", "-".repeat(66));
    let msg1 = signer
        .sign_and_encrypt(
            b"Hello! First BUTL v1.2 message.",
            &rx.public_key_bytes(),
            &rx.address,
        )
        .unwrap();
    println!("Encrypted payload: {} bytes", msg1.encrypted_payload.len());
    let s1pk = hex::decode(&msg1.header.sender_pubkey).unwrap();
    let mut r1 = gate.check_header(&msg1.header, None);
    if r1.gate_passed() {
        if let Some(pt) = gate.accept_payload(&msg1.header, &msg1.encrypted_payload, &mut r1) {
            println!("Decrypted: \"{}\"", String::from_utf8_lossy(&pt));
        }
    }
    println!("\n{}", r1.summary());

    // -- Message 2: Chained --
    println!("\n{}\nMESSAGE 2 (Chained)\n", "-".repeat(66));
    let msg2 = signer
        .sign_and_encrypt(
            b"Second message, chained to the first.",
            &rx.public_key_bytes(),
            &rx.address,
        )
        .unwrap();
    let s2pk = hex::decode(&msg2.header.sender_pubkey).unwrap();
    let mut r2 = gate.check_header(&msg2.header, Some(&s1pk));
    if r2.gate_passed() {
        if let Some(pt) = gate.accept_payload(&msg2.header, &msg2.encrypted_payload, &mut r2) {
            println!("Decrypted: \"{}\"", String::from_utf8_lossy(&pt));
        }
    }
    println!("\n{}", r2.summary());

    // -- Tampered payload --
    println!("\n{}\nTAMPERED PAYLOAD\n", "-".repeat(66));
    let mut tampered = msg2.encrypted_payload.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0xFF;
    let mut rt = gate.check_header(&msg2.header, Some(&s1pk));
    if rt.gate_passed() {
        let pt = gate.accept_payload(&msg2.header, &tampered, &mut rt);
        println!("Payload accepted: {}", pt.is_some());
    }
    println!("\n{}", rt.summary());

    // -- Wrong receiver --
    println!("\n{}\nWRONG RECEIVER\n", "-".repeat(66));
    let wrong = BUTLKeypair::from_secret(&sha256(b"wrong-person")).unwrap();
    let wg = BUTLGate::new(
        wrong,
        StubBlockchain,
        144,
        ProofOfSatoshiConfig::default(),
    );
    let rw = wg.check_header(&msg1.header, None);
    println!(
        "Gate: {}",
        if rw.gate_passed() { "OPEN" } else { "CLOSED" }
    );
    println!("\n{}", rw.summary());

    // -- Proof of Satoshi enabled --
    println!(
        "\n{}\nPROOF OF SATOSHI ENABLED (10,000 sat)\n",
        "-".repeat(66)
    );
    let pos_gate = BUTLGate::new(
        rx.clone(),
        StubBlockchain,
        144,
        ProofOfSatoshiConfig {
            enabled: true,
            min_satoshis: 10000,
        },
    );
    let msg3 = signer
        .sign_and_encrypt(b"PoS test", &rx.public_key_bytes(), &rx.address)
        .unwrap();
    let mut rp = pos_gate.check_header(&msg3.header, Some(&s2pk));
    if rp.gate_passed() {
        if let Some(pt) =
            pos_gate.accept_payload(&msg3.header, &msg3.encrypted_payload, &mut rp)
        {
            println!("Decrypted: \"{}\"", String::from_utf8_lossy(&pt));
        }
    }
    println!("\n{}", rp.summary());

    // -- Address chain --
    println!("\n{}\nADDRESS CHAIN\n", "-".repeat(66));
    println!("Thread ID: {}", signer.keychain.thread_id());
    for (i, a) in signer.keychain.history().iter().enumerate() {
        println!(
            "  Msg {}: {}  {}",
            i,
            a,
            if i == 0 { "(genesis)" } else { "(chained)" }
        );
    }

    println!(
        "\n{}\n  Demo complete.\n{}",
        "=".repeat(66),
        "=".repeat(66)
    );
}
