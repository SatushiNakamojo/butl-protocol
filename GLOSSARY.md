# Glossary

## BUTL Protocol — Definitions of All Terms

This glossary defines every term specific to the BUTL Protocol. General cryptographic terms are included where their BUTL-specific usage differs from or extends the general definition.

-----

## A

**AAD (Additional Authenticated Data)**
Extra data included in AES-256-GCM encryption that is authenticated but not encrypted. In BUTL, the AAD is the concatenation of the sender’s Bitcoin address and the receiver’s Bitcoin address (`sender_addr || receiver_addr`). This binds the ciphertext to both parties — if an attacker tries to redirect the encrypted payload to a different receiver, the GCM authentication check fails.

**Address**
A Bitcoin address derived from a BUTL public key through SHA-256, RIPEMD-160, and Base58Check encoding (for P2PKH addresses starting with “1”) or Bech32 encoding (for SegWit addresses starting with “bc1”). The address is the human-readable form of a BUTL identity. It is shorter than the public key, includes error-detection checksums, and is what the Bitcoin blockchain is indexed by for Proof of Satoshi balance checks.

**Address Chaining**
The mechanism where each BUTL message uses a fresh Bitcoin address, with a chain proof cryptographically linking the new address to the previous one. This provides three properties simultaneously: forward secrecy (compromising one key affects only one message), privacy (third-party observers see unrelated addresses), and verifiable continuity (the receiver can confirm the same entity sent all messages in the chain).

**AES-256-GCM**
Advanced Encryption Standard with 256-bit keys in Galois/Counter Mode. The default authenticated encryption algorithm used by BUTL for payload encryption. Provides both confidentiality (only the intended recipient can read the data) and integrity (any modification to the ciphertext is detected) in a single operation. Standardized in NIST SP 800-38D.

-----

## B

**Balance Check**
See Proof of Satoshi.

**Base58Check**
The encoding format used for legacy Bitcoin addresses (those starting with “1”). Converts binary data to a human-readable string using 58 characters (excluding 0, O, I, and l to avoid visual ambiguity), with a 4-byte SHA-256d checksum for error detection. A single mistyped character will fail the checksum.

**Bech32**
The encoding format used for SegWit Bitcoin addresses (those starting with “bc1”). Defined in BIP-173. Uses a different character set and error-detection algorithm than Base58Check. Bech32m (BIP-350) is the updated version for taproot addresses.

**BIP (Bitcoin Improvement Proposal)**
A formal proposal for changes or additions to the Bitcoin protocol or ecosystem. BIPs referenced by BUTL include BIP-32 (hierarchical deterministic wallets), BIP-39 (mnemonic seed phrases), BIP-173 (Bech32 encoding), and BIP-350 (Bech32m encoding).

**Block Hash**
The SHA-256d hash of a Bitcoin block header. Included in the BUTL header (`BUTL-BlockHash`) alongside the block height to allow the receiver to verify that the referenced block actually exists on the canonical chain.

**Block Height**
The number of blocks in the Bitcoin blockchain between the genesis block and a given block. BUTL uses block height as a freshness timestamp — the receiver checks that the referenced block height is within a configurable window (default: 144 blocks, approximately 24 hours) of the current chain tip. This prevents replay attacks.

**BlockchainProvider**
A trait (Rust) or interface (Python) in the BUTL reference implementations that abstracts the connection to the Bitcoin blockchain. Production deployments implement this with a real Bitcoin node, Electrum server, or API. The reference implementations include a `StubBlockchain` for development and testing.

**BUTL**
Bitcoin Universal Trust Layer. The protocol defined in this repository.

**BUTL Gate**
The verification boundary between header checks and payload delivery. The gate consists of steps 1 through 6 of the verification sequence. If all enabled checks pass, the gate is OPEN and payload delivery begins. If any check fails, the gate is CLOSED and the connection is dropped with nothing saved to the device.

**BUTL Header**
The cleartext portion of a BUTL message. Contains all metadata required for the verification gate: sender and receiver identity (address + public key), signature, block height and hash, payload hash, chain proof, encryption algorithm, and optional fields (ephemeral key, nonce, timestamp, thread ID, sequence number, payload size). The header is transmitted in Phase 1 and verified before any payload data is downloaded.

**BUTL-ID**
A BUTL identity consisting of three components: a private key (the secret), a public key (the cryptographic tool), and a Bitcoin address (the human-readable identifier). All three are mathematically linked. The public key and address are derived from the private key through one-way functions.

**BUTL Message**
A complete BUTL transmission consisting of a header (cleartext) and an encrypted payload. The header is delivered in Phase 1 for verification. The payload is delivered in Phase 2 only if the gate passes.

-----

## C

**Canonical Signing Payload**
The deterministic string constructed from BUTL header fields that is hashed with SHA-256 and signed with ECDSA. The exact format ensures that the sender and receiver compute the same hash for signature creation and verification. Fields are separated by newline characters (`\n`) in a fixed order: Version, Sender, SenderPubKey, Receiver, ReceiverPubKey, BlockHeight, BlockHash, PayloadHash, PrevAddr, Nonce.

**Chain Proof**
An ECDSA signature where the previous message’s private key signs the SHA-256 hash of the current message’s sender address. This proves that the entity controlling the previous address also controls the new address, without revealing the previous private key. Carried in the `BUTL-ChainProof` header field.

**Compressed Public Key**
A 33-byte representation of a secp256k1 public key point. The first byte is `0x02` (if the y-coordinate is even) or `0x03` (if odd), followed by the 32-byte x-coordinate. The full y-coordinate is recovered from the curve equation during decompression. All BUTL public keys are transmitted in compressed form (66 hex characters).

**Confidence Level**
An assessment of the probability that BUTL functions as specified for a correctly implemented deployment. Without Proof of Satoshi: 99.7% (pure math, zero external dependencies, 0.3% residual = implementation bug risk). With Proof of Satoshi: 97% (blockchain query introduces an external trust dependency).

-----

## D

**Defensive Patent Pledge**
An irrevocable commitment by all contributors that they will never file patents on BUTL methods, will provide prior art evidence to defeat third-party patents, and will never sell patent rights to litigation entities. This pledge survives any transfer of copyright, corporate acquisition, or change of maintainership.

**Double SHA-256 (SHA-256d)**
Applying SHA-256 twice: `SHA-256(SHA-256(data))`. Used in Bitcoin for block hashing, transaction hashing, and the checksum in Base58Check encoding.

**Dual License**
BUTL is licensed under both MIT and Apache 2.0, with the user choosing which license applies to their use. The PATENTS file and Defensive Patent Pledge apply to all users regardless of license choice.

-----

## E

**ECDH (Elliptic Curve Diffie-Hellman)**
A key agreement protocol where two parties each use their private key and the other party’s public key to compute an identical shared secret. In BUTL: `shared_secret = SHA-256((my_private_key × their_public_key).x)`. The shared secret becomes the AES-256-GCM encryption key.

**ECDSA (Elliptic Curve Digital Signature Algorithm)**
A digital signature algorithm using elliptic curve cryptography. In BUTL, ECDSA on secp256k1 is used for message authentication (the sender signs the canonical payload) and chain proof verification (the previous key signs the current address). The security relies on the Elliptic Curve Discrete Logarithm Problem.

**ECDLP (Elliptic Curve Discrete Logarithm Problem)**
The mathematical problem underlying ECDSA and ECDH security: given a point `Q = k × G` on an elliptic curve, it is computationally infeasible to determine `k` (the private key) from `Q` (the public key) and `G` (the generator point). Breaking this on secp256k1 would break Bitcoin and BUTL simultaneously.

**Encrypted Payload**
The message body after AES-256-GCM encryption. Binary format: `IV (12 bytes) || ciphertext (variable) || GCM tag (16 bytes)`. Transmitted only in Phase 2, after the gate passes.

**Ephemeral Key**
A temporary keypair generated for a single ECDH key exchange, then discarded. When the `BUTL-EphemeralPubKey` header field is present, the receiver uses the ephemeral key (rather than the sender’s signing key) for ECDH. This provides enhanced forward secrecy by separating the signing key from the encryption key.

-----

## F

**Forward Secrecy**
The property that compromising the current key does not compromise past communications. BUTL achieves this through address chaining (each message uses a new keypair) and optional ephemeral keys for ECDH.

**Freshness Window**
The maximum age (in blocks) that a BUTL message’s block height reference can be and still be accepted by the receiver. Default: 144 blocks (approximately 24 hours). A tighter window (e.g., 6 blocks for high-security APIs) rejects messages faster. A wider window (e.g., 1008 blocks for delay-tolerant systems) allows older messages.

-----

## G

**Gate**
See BUTL Gate.

**GateReport**
A data structure in the BUTL reference implementations that records the result of each verification step (PASS, FAIL, or SKIP), along with any error messages. Provides `gate_passed` (did steps 1-6 succeed?) and `fully_verified` (did all 8 steps succeed?) properties.

**GCM Tag**
The 16-byte authentication tag produced by AES-256-GCM. Verifies both the integrity and authenticity of the ciphertext and the AAD. If any bit of the ciphertext, the AAD, or the tag itself is modified, decryption fails.

**Genesis Message**
The first message in a BUTL address chain. It has an empty `BUTL-PrevAddr` field and no chain proof. It establishes the ThreadID for the conversation.

-----

## H

**Handoff**
A structured JSON blob that BUTL-Contacts passes to a spoke application (Chat, VideoChat, FileTransfer, Login). Contains the sender’s identity, the receiver’s identity and chain state, and gate configuration. The handoff enables spoke apps to operate without implementing their own identity management or contact lookup.

-----

## K

**Keychain**
A sequence of keypairs derived deterministically from a master seed. Each call to `next_keypair()` produces a new keypair at the next index, retains the previous keypair for chain proof creation, and records the address in the chain history. Implemented as `BUTLKeychain` in the reference implementations.

**Keypair**
A private key and its corresponding public key on the secp256k1 curve. In BUTL, a keypair also includes the derived Bitcoin address. Implemented as `BUTLKeypair` in the reference implementations.

-----

## M

**Master Seed**
The 32-byte random value from which all keypairs in a keychain are derived. The seed can be represented as a BIP-39 mnemonic phrase (12 or 24 words) for human-readable backup. The same seed always produces the same sequence of keypairs — this is what makes identity recovery from a seed phrase possible.

**MVP (Minimum Viable Prototype)**
A single Python file (`butl_mvp_v12.py`) with zero external dependencies that proves all 8 BUTL protocol claims plus 4 Proof of Satoshi scenarios. It implements secp256k1 elliptic curve math, ECDSA, ECDH, and authenticated encryption entirely in pure Python.

-----

## N

**Nonce**
A random value included in the BUTL header (`BUTL-Nonce`) for additional anti-replay protection. Combined with block height freshness, it ensures that even two messages sent within the same block cannot be confused or replayed.

-----

## P

**P2PKH (Pay-to-Public-Key-Hash)**
The Bitcoin address format starting with “1”. Constructed as `Base58Check(0x00 || RIPEMD-160(SHA-256(compressed_pubkey)))`. The default address format used by BUTL.

**Patent Grant**
The `PATENTS.md` file in the BUTL repository. Provides an explicit, perpetual, worldwide, royalty-free, irrevocable patent license covering all BUTL methods. Includes a retaliation clause: anyone who sues a BUTL user for patent infringement loses their own license. Applies to all users regardless of which license (MIT or Apache 2.0) they choose.

**Payload**
The encrypted message body. In BUTL, the payload is always encrypted with AES-256-GCM, and its SHA-256 hash is included in the header as `BUTL-PayloadHash` for integrity verification.

**Payload Hash**
The SHA-256 hash of the encrypted payload, carried in the `BUTL-PayloadHash` header field. The receiver computes the hash of the received encrypted payload and compares it to this value. A mismatch means the payload was tampered with in transit.

**Phase 1**
The header delivery and verification phase. Only the BUTL header is transmitted. The receiver runs gate checks (steps 1-6) on the header alone. No payload data is involved. If the gate fails, Phase 2 never occurs.

**Phase 2**
The payload delivery phase. Occurs only if Phase 1 (the gate) passes. The encrypted payload is transmitted, hash-verified (step 7), and decrypted (step 8). If either step fails, the payload is zeroed from memory immediately.

**Prior Art**
Published, timestamped evidence that a method or invention was publicly known before a patent application was filed. The BUTL repository, its Wayback Machine archives, and optional Bitcoin blockchain timestamps establish prior art that prevents anyone from patenting BUTL’s methods after the publication date.

**Private Key**
A 256-bit integer on the secp256k1 curve. The root of a BUTL identity. Everything else — the public key, the Bitcoin address, the ability to sign, the ability to decrypt — is derived from or depends on the private key. If the private key is lost, the identity is gone permanently. If the private key is compromised, someone else can impersonate the identity.

**Proof of Satoshi (PoS)**
An optional BUTL verification step (Step 5) where the receiver checks that the sender's Bitcoin address holds at least a configurable minimum number of satoshis. Provides sybil resistance by creating an economic cost to identity creation. Configured by the receiver via two settings: `pos_enabled` (boolean toggle) and `pos_min_satoshis` (integer slider, range 1 — 2,100,000,000,000,000, i.e. 1 sat to 21,000,000 BTC).

**Public Key**
A point on the secp256k1 elliptic curve derived from the private key via scalar multiplication (`public_key = private_key × G`). Used for ECDSA signature verification, ECDH shared secret computation, and chain proof verification. Transmitted in compressed form (33 bytes / 66 hex characters) in the `BUTL-SenderPubKey` and `BUTL-ReceiverPubKey` header fields.

**Pure Cryptography Mode**
BUTL operating with Proof of Satoshi disabled. The verification gate runs steps 1-4 and 6 only — all of which are local computations with zero external dependencies. Every check is pure mathematics: string comparison, ECDSA verification, integer comparison, and SHA-256 hashing.

-----

## R

**RIPEMD-160**
A 160-bit cryptographic hash function used in Bitcoin address derivation: `RIPEMD-160(SHA-256(public_key))`. The result is the 20-byte payload of a P2PKH address.

-----

## S

**secp256k1**
The specific elliptic curve used by Bitcoin and BUTL. Defined by the equation `y² = x³ + 7` over a 256-bit prime field. The curve parameters (prime, order, generator point) are public constants. The same curve secures all Bitcoin transactions and is the most battle-tested elliptic curve in existence.

**Seed Phrase**
A sequence of 12 or 24 English words (defined by BIP-39) that encodes a master seed. The same words in the same order always produce the same seed, which produces the same keypair sequence. The seed phrase is the recommended backup format for BUTL identities — write the words on paper, store them safely, and the identity can be restored on any device at any time.

**SHA-256**
Secure Hash Algorithm with 256-bit output. Defined in FIPS 180-4. Used throughout BUTL for hashing the canonical signing payload, computing the payload hash, deriving Bitcoin addresses (as the first step before RIPEMD-160), computing ECDH shared secrets, generating ThreadIDs, and producing chain proof hashes.

**Signer**
The sender-side component that constructs and signs BUTL messages. Takes a plaintext body, a receiver’s public key and address, and produces a complete BUTLMessage (header + encrypted payload). Implemented as `BUTLSigner` in the reference implementations.

**Spoke Application**
An application in the BUTL ecosystem that receives a handoff from BUTL-Contacts and handles a specific communication domain: Chat, VideoChat, FileTransfer, Login, Email, Sign. Each spoke app focuses on its domain and relies on Contacts for identity management and contact lookup.

**Stub Blockchain**
A placeholder implementation of the BlockchainProvider interface that returns hardcoded values for block height, block hash, and balance. Used in development, testing, and the MVP. Must be replaced with a real blockchain connection for production.

**Sybil Attack**
An attack where an adversary creates many fake identities to gain disproportionate influence. In a BUTL context: creating thousands of valid keypairs (each with valid signatures, encryption, and chain proofs) at zero cost to spam, flood, or manipulate a system.

**Sybil Resistance**
The ability to make identity creation costly. BUTL’s Proof of Satoshi provides sybil resistance by requiring each identity to hold a minimum Bitcoin balance. When Proof of Satoshi is disabled, BUTL does not provide sybil resistance.

-----

## T

**ThreadID**
The identifier for a BUTL conversation thread. Computed as `SHA-256(Address_0)` where `Address_0` is the first sender address in the chain (the genesis message address). The ThreadID remains constant across all messages in the chain, even as the sender’s address changes with every message.

**Two-Phase Delivery**
BUTL’s security model where the header is transmitted and verified first (Phase 1), and the payload is transmitted only after the gate passes (Phase 2). This ensures that no payload data — including potential malware, phishing content, or oversized files — reaches the receiving device until all cryptographic checks have succeeded.

-----

## V

**Verify-Before-Download**
The BUTL principle that no payload data touches the receiving device until all header verification checks pass. The inverse of the traditional “download first, verify later” model. This prevents entire categories of attacks: malware delivery, resource exhaustion, phishing content rendering, and supply-chain payload injection.

-----

## W

**.well-known/butl.json**
A standard HTTPS endpoint for publishing BUTL receiver public keys. Located at `https://{domain}/.well-known/butl.json`. Returns a JSON object containing the receiver’s Bitcoin address, compressed public key, and configuration preferences (Proof of Satoshi requirements, freshness window, supported protocol versions). Enables sender-side key discovery without any centralized directory. Follows the IETF `.well-known` URI convention (RFC 8615).

-----

*Term not listed here? Open a GitHub issue with the label `documentation`.*