# Roadmap

## BUTL Protocol — Planned Development

This document outlines the development trajectory of the BUTL Protocol from its current state through future versions. It is a living document that evolves based on community feedback, technical discoveries, and real-world adoption.

-----

## Where We Are Now

### v1.2 — Initial Public Release (March 2026)

**Status: Complete. Published.**

The foundation is built. The protocol is specified, implemented, proven, documented, and legally protected.

|Deliverable                                          |Status               |
|-----------------------------------------------------|---------------------|
|Protocol Specification v1.2                          |Complete             |
|Python reference implementation (866 lines)          |Complete             |
|Rust reference implementation                        |Complete             |
|Zero-dependency MVP (8 proofs + 4 PoS scenarios)     |Complete and verified|
|Proof of Satoshi: optional toggle + slider           |Complete             |
|White paper                                          |Complete             |
|11 protocol diagrams                                 |Complete             |
|Dual license (MIT OR Apache 2.0)                     |Complete             |
|Patent grant + Defensive Patent Pledge               |Complete             |
|AI context transfer documents                        |Complete             |
|Beginner guides (Python, Rust, zero-experience proof)|Complete             |
|FAQ, glossary, security policy                       |Complete             |
|AI generation and collaboration disclosures          |Complete             |

-----

## Where We Are Going

### Phase 1 — BUTL-Contacts (Target: Q3-Q4 2026)

**The identity hub for the BUTL ecosystem.**

BUTL-Contacts is the first real-world application built on the protocol. It serves as the central identity manager, contact book, and handoff engine that every future BUTL application depends on.

|Component        |Description                                                                                                                                            |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
|Identity Manager |Create, encrypt, store, backup (BIP-39 seed phrase), and restore BUTL identities                                                                       |
|Contact Book     |Add, view, edit, search, and delete contacts with BUTL fields (address, pubkey, chain state) and freeform personal fields (email, phone, website, etc.)|
|Handoff Engine   |Standardized JSON handoff protocol for passing BUTL context to spoke applications                                                                      |
|Settings         |Proof of Satoshi toggle/slider, freshness window, block height source configuration                                                                    |
|Encrypted Storage|All identity and contact data encrypted at rest (Argon2id + AES-256-GCM)                                                                               |

Platform: CLI-first (Python). Designed for a GUI layer to be added later without changing core logic.

The internal architecture separates identity management into its own module with a clean API boundary, enabling future extraction into a standalone BUTL-ID service for enhanced security isolation.

### Phase 2 — BUTL-Chat (Target: Q4 2026 - Q1 2027)

**The first spoke application. Proves the protocol works between two real people in real time.**

BUTL-Chat receives a handoff from BUTL-Contacts and enables BUTL-signed, BUTL-encrypted messaging between two parties. Every message displays the full 8-step gate verification in real time.

|Feature            |Description                                                                                              |
|-------------------|---------------------------------------------------------------------------------------------------------|
|Real-time messaging|Two terminals (or two devices on a LAN) exchange BUTL messages over TCP                                  |
|Full gate display  |Every message shows steps 1-8: structure, receiver match, signature, freshness, PoS, chain, hash, decrypt|
|Address chaining   |Each message uses a fresh address, chained to the previous, with automatic state updates                 |
|Handoff integration|Receives BUTL context from BUTL-Contacts, no manual key entry required                                   |

Platform: CLI-first (Python). TCP transport for the initial version.

-----

### v1.3 — Production Hardening (Target: H1 2027)

**Making BUTL ready for real-world deployment.**

#### Key Revocation

The most-requested missing feature. Enables a sender to signal that a key or address chain has been compromised.

- `BUTL-Revocation` header for signaling compromised keys
- Signed revocation of a ThreadID using a designated revocation key
- Receiver stores revocation list and rejects messages from revoked chains
- Revocation propagation mechanism for relay servers

#### Production Libraries

Packaged libraries that developers can install with one command.

|Language               |Package Manager|Install Command   |
|-----------------------|---------------|------------------|
|Python                 |PyPI           |`pip install butl`|
|Rust                   |crates.io      |`cargo add butl`  |
|JavaScript / TypeScript|npm            |`npm install butl`|

Each package includes the full reference implementation, type definitions, and documentation.

#### Formal Test Suite

- Deterministic test vectors with exact expected byte-level outputs
- Cross-language test runner (Python tests validate Rust output and vice versa)
- CI/CD integration (GitHub Actions running the full test suite on every commit and pull request)
- Fuzz testing for header parsing and cryptographic operations

#### Key Discovery Reference Implementation

- Python Flask/FastAPI server serving `.well-known/butl.json`
- Rust Actix/Axum server serving `.well-known/butl.json`
- DNS TXT record generation utility
- Key discovery client library for both languages

#### Real Blockchain Integration

- Bitcoin Core RPC adapter (BlockchainProvider implementation)
- Electrum server adapter
- mempool.space REST API adapter
- Block height caching with configurable TTL
- SPV proof verification for offline-capable balance checks

-----

### v2.0 — Protocol Evolution (Target: 2027-2028)

**Major protocol features that require a version bump.**

#### Multi-Party Support

- `BUTL-MultiSig` header for messages requiring multiple signers
- Group messaging with shared ThreadIDs and group key agreement
- Threshold signatures (m-of-n signing) for organizational identities
- Group address chaining (multiple participants, one chain)

#### Post-Quantum Migration Path

Quantum computers capable of running Shor’s algorithm would break secp256k1. BUTL must be ready.

- `BUTL-PostQuantum` header for algorithm negotiation
- Hybrid signatures: ECDSA + Dilithium (or Falcon) — both must verify
- Hybrid key exchange: ECDH + ML-KEM (Kyber) — both contribute to shared secret
- Backward compatibility with v1.x (hybrid mode optional, not required)
- Migration guide for existing deployments

#### BUTL-ID as a Standalone Service

Extract identity management from BUTL-Contacts into its own process/service.

- BUTL-ID manages exactly one thing: the encrypted keypair
- API: `sign(hash)`, `ecdh(their_pubkey)`, `public_info()`, `backup()`, `restore()`
- Private key never leaves BUTL-ID’s process boundary
- Spoke apps (Chat, Video, Login) call BUTL-ID directly for cryptographic operations
- Enables hardware security module (HSM) and secure enclave integration

#### Formal Security Audit

- Engage a professional cryptography firm to audit both the protocol design and reference implementations
- Publish the complete audit report publicly
- Address all findings before v2.0 final release
- Re-audit after significant changes

#### IETF RFC Submission

- Format the specification as an Internet-Draft
- Submit to the IETF for consideration as an Informational or Standards Track RFC
- Engage with the CFRG (Crypto Forum Research Group) for cryptographic review
- Engage with relevant working groups for application binding standards

-----

### v3.0+ — Ecosystem Expansion (2028+)

**Broadening BUTL’s reach across platforms and use cases.**

#### Hardware Integration

- BUTL implementation for secure enclaves (ARM TrustZone, Apple Secure Enclave, Intel SGX)
- BUTL bootloader reference implementation (bare-metal Rust)
- Smart card and hardware key support (YubiKey, COLDCARD, Ledger, Trezor)
- BUTL-ID running entirely on hardware — private key never in software memory

#### Spoke Applications

Each spoke receives a handoff from BUTL-Contacts and focuses on a single domain.

|Application      |Description                                                     |
|-----------------|----------------------------------------------------------------|
|BUTL-VideoChat   |ECDH-encrypted video/voice calls with BUTL identity verification|
|BUTL-FileTransfer|Encrypted file transfer with verify-before-download             |
|BUTL-Login       |Browser extension for passwordless website authentication       |
|BUTL-Email       |Email client plugin adding BUTL headers to SMTP                 |
|BUTL-Sign        |Document signing and notarization with blockchain timestamps    |

#### Protocol Extensions

- `BUTL-Stream` header for streaming media (video, voice, live data)
- Broadcast/multicast mode (one sender, many receivers, each with unique ECDH)
- Offline mesh networking support (delay-tolerant BUTL for low-connectivity environments)
- Relay server specification for store-and-forward delivery

#### Platform SDKs

|Platform|Language  |Distribution       |
|--------|----------|-------------------|
|iOS     |Swift     |CocoaPods / SPM    |
|Android |Kotlin    |Maven Central      |
|Web     |TypeScript|npm                |
|Desktop |Rust      |Standalone binary  |
|Embedded|C         |Header-only library|

#### Ecosystem Infrastructure

- BUTL API gateway: drop-in replacement for API key authentication
- BUTL certificate transparency log: public record of address chain events
- BUTL directory service: optional, federated public key discovery beyond `.well-known`

-----

## What Will Not Change

Regardless of version, the following properties are permanent:

- **Self-sovereign identity.** Your identity is your key. No registration, no permission, no third party. Ever.
- **Pure math core.** Without Proof of Satoshi, the protocol is 100% local computation with zero external trust dependencies. This property is non-negotiable.
- **Verify-before-download.** No payload touches the device until all checks pass.
- **Open protocol.** Dual licensed (MIT OR Apache 2.0). Patent grant applies to all users. Cannot be patented. Cannot be closed.
- **Bitcoin-anchored.** secp256k1, SHA-256, block heights. The trust anchor is the most battle-tested cryptographic infrastructure in existence.

-----

## How to Influence the Roadmap

Open a GitHub issue with the label `roadmap` and describe:

- **What** capability is needed
- **Why** it matters (what problem does it solve, who benefits)
- **How** it might work (optional, but helpful)

Roadmap priorities are driven by community demand, technical feasibility, and alignment with the protocol’s core principles. Every suggestion is read and considered.

-----

*The protocol is built. The ecosystem is next. Every phase makes the trust layer stronger.*