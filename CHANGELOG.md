# Changelog

All notable changes to the BUTL Protocol are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/). BUTL uses [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH` where MAJOR is breaking protocol changes, MINOR is new features, and PATCH is fixes.

-----

## v1.2.1 — 2026-03-15

### Documentation

- Clarified Proof of Satoshi slider range: 1 sat — 21,000,000 BTC (was 1 sat — 1 BTC)
- Added logarithmic slider recommendation for UI implementations
- No protocol change. No wire format change. No code change.

-----

## [1.2.0] — 2026-03-14

### Protocol Changes

- **Proof of Satoshi is now OPTIONAL.** The balance check (Step 5) is a receiver-side toggle with a configurable threshold slider. When disabled, BUTL operates as a pure-math cryptographic trust layer with zero external dependencies. When enabled, the receiver sets the minimum satoshi requirement (1 sat — 21,000,000 BTC).
- Added `pos_enabled` and `pos_min_satoshis` receiver configuration parameters.
- Added Section 6 to the specification: Proof of Satoshi — dedicated section with recommended thresholds per use case.
- Added Section 8.1 to the specification: Pure Cryptography Mode — documents the gate flow without Proof of Satoshi.
- Verification sequence Step 5 is now marked OPTIONAL in the specification.

### Licensing

- **Dual licensed: MIT OR Apache 2.0, at user’s option.** Replaces the previous Apache 2.0 only approach. Follows the same pattern used by Rust, serde, and the broader Rust ecosystem.
- Added `LICENSE.md` — dual license overview explaining which to choose and why.
- Added `LICENSE-MIT.md` — full MIT License text.
- Added `LICENSE-APACHE.md` — full Apache License 2.0 text with built-in patent grant (Section 3) and patent retaliation clause.
- Updated `PATENTS.md` — explicit patent grant now states it applies to all users regardless of license choice.
- Updated `DEFENSIVE-PATENT-PLEDGE.md` — now includes a “Relationship to Licensing” section explaining the triple/double protection layers.
- Added `NOTICE.md` — attribution, cryptographic primitives, third-party dependencies, standards referenced, and acknowledgments.
- Added `legal/DUAL_LICENSE_ANALYSIS.md` — analysis of MIT vs Apache 2.0 vs both, with recommendation.

### Specification

- Added `spec/HEADER_REGISTRY.md` — formal IANA-style registry of all BUTL-* header fields with types, formats, and version introduced.
- Added `spec/VERSIONING.md` — version negotiation, backward compatibility rules, forward compatibility, and 24-month migration strategy.
- Added `spec/TEST_VECTORS.md` — 7 deterministic test cases for cross-implementation verification.
- Added `spec/WELL_KNOWN_BUTL_SPEC.md` — `.well-known/butl.json` standard for receiver public key discovery, including DNS TXT record fallback and IANA registration template.

### Implementations

- Added `python/butl_v12.py` — complete Python reference implementation (866 lines). Classes: BUTLKeypair, BUTLKeychain, BUTLHeader, BUTLSigner, BUTLGate, GateReport, ProofOfSatoshiConfig. Dependencies: ecdsa, pycryptodome.
- Added `rust/butl_v12.rs.md` — complete Rust reference implementation with Cargo.toml. Structs mirror Python API. Trait: BlockchainProvider with StubBlockchain.
- Updated `mvp/butl_mvp_v12.py` — Python MVP now proves all 8 claims plus 4 Proof of Satoshi sub-scenarios (disabled, enabled+passes, enabled+threshold too high, enabled+zero balance). Zero external dependencies.
- Updated `mvp/butl_mvp_v12_rust.md` — Rust MVP with same 4 Proof of Satoshi sub-scenarios.

### Documentation

- Added `BUTL_WHITE_PAPER.md` — white paper in the style of the original Bitcoin white paper. 12 sections plus 8 references.
- Added `BUTL_ONE_PAGER.md` — 60-second protocol overview for non-technical audiences and social sharing.
- Added `BUTL_DIAGRAMS.md` — 11 ASCII protocol diagrams covering the full architecture, message flow, key generation, ECDH, address chaining, verification gate, header structure, payload format, Proof of Satoshi, application bindings, and security properties.
- Added `KEY_IS_IDENTITY_WARNING.md` — critical warning that the private key is the identity, with 5 protection rules, backup instructions, and compromise response procedures.
- Added `WHY_ADDRESS_AND_PUBKEY.md` — explains why BUTL-ID contains both a Bitcoin address and a public key, and what each is used for.
- Added `FAQ.md` — 20 frequently asked questions covering general, technical, Proof of Satoshi, comparison, and legal topics.
- Added `GLOSSARY.md` — definitions of all BUTL-specific terms.
- Added `ROADMAP.md` — planned development: v1.3 (production libraries, key revocation), v2.0 (multi-party, post-quantum, IETF RFC), v3.0+ (hardware, streaming, mobile SDKs).
- Added `docs/BUTL_PURE_MATH_PROOF.md` — step-by-step proof that BUTL without Proof of Satoshi is 100% local math with zero external trust dependencies.
- Added `docs/CONFIDENCE_WITHOUT_POS.md` — 99.7% confidence analysis with full justification.
- Added `docs/PROOF_OF_SATOSHI_BENEFITS.md` — 7 benefits of making Proof of Satoshi optional with a configurable slider.
- Added `docs/DEVELOPER_WARNING_POS_DISABLED.md` — warning that disabling Proof of Satoshi removes anti-sybil protection, with 3 attack scenarios and a developer checklist.
- Added `docs/BUTL_BOOT_LEVEL_INTEGRATION.md` — how BUTL can secure the firmware/bootloader/OS trust chain.
- Added `docs/BUTL_ECOSYSTEM_ARCHITECTURE.md` — Contacts-as-hub, apps-as-spokes ecosystem architecture.
- Added `docs/INTEGRATION_EXAMPLES.md` — copy-paste code snippets for passwordless login, authenticated email, REST API authentication, IoT device registration, and encrypted file transfer.
- Added `docs/PROOF_OF_CONCEPT_RECOMMENDATION.md` — analysis of the best first real-world proof of concept.

### Guides

- Added `python/README.md` — beginner’s guide to using the Python reference implementation (zero experience required).
- Added `rust/README.md` — beginner’s guide to using the Rust reference implementation (zero experience required).
- Added `mvp/HOW_TO_PROVE_BUTL_WORKS.md` — step-by-step guide for complete beginners to prove the protocol works on their own computer.
- Added `mvp/VERIFICATION_CHECKLIST.md` — printable checkbox form for recording proof results.
- Added `mvp/QUICK_REFERENCE.md` — one-page command card.
- Added `legal/GITHUB_GUIDE.md` — complete beginner’s guide to managing the GitHub repository.
- Added `legal/GITHUB_OPEN_SOURCE_GUIDE.md` — step-by-step guide to publishing on GitHub with maximum patent protection.

### AI Context

- Added `ai/AI_BRIEFING_DOCUMENT.md` — complete context transfer document (17 sections) for bringing any AI instance up to speed on BUTL.
- Added `ai/AI_QUICK_START_PROMPT.md` — compact 32-line version for fast AI context loading.
- Added `ai/AI_CODE_GENERATION_REFERENCE.md` — exact data structures, algorithms, and byte-level formats for AI code generation in any language.

### Transparency

- Added `AI_GENERATION_DISCLOSURE.md` — discloses that all documents and code were initially generated by Claude (Anthropic).
- Added `AI_COLLABORATION_DISCLOSURE.md` — discloses additional collaboration with Grok (xAI) and ChatGPT (OpenAI).

### Repository

- Added `CONTRIBUTING.md` — contributor license agreement, submission process, code style, and protocol change requirements. Written for a project that may have only one maintainer.
- Added `CODE_OF_CONDUCT.md` — community standards, enforcement, and maintainer commitments. Written for a project that may have only one maintainer.
- Added `SECURITY.md` — vulnerability reporting, scope, known limitations with mitigations, cryptographic foundation, responsible disclosure timeline.
- Added `.gitignore` — Python, Rust, OS, IDE, and secrets exclusions.

-----

## [1.1.0] — 2026-03-07

### Added

- End-to-end encryption via ECDH key agreement + AES-256-GCM.
- Receiver public key fields (BUTL-Receiver, BUTL-ReceiverPubKey).
- Verify-before-download gate (two-phase delivery model).
- Ephemeral key support (BUTL-EphemeralPubKey) for enhanced forward secrecy.
- BUTL-PayloadSize field for pre-download validation.
- BUTL-EncAlgo field.
- 8-step verification sequence (6 gate checks + 2 payload checks).
- Python reference implementation (butl_v11.py).
- Rust reference implementation (butl_v11.rs).
- Minimum Viable Prototype with 8 cryptographic proofs.
- Complete documentation and implementation guides.

-----

## [1.0.0] — 2026-03-07

### Added

- Initial BUTL protocol specification.
- SHA-256 body integrity.
- Bitcoin address identity (secp256k1 + Base58Check).
- ECDSA signature verification.
- Block height freshness checking (144-block window).
- Balance gate (≥1 satoshi sybil resistance).
- Address chaining with chain proofs.
- Verifiable threaded messages (ThreadID, SeqNum).
- Canonical signing payload format.
- Application bindings (email, HTTP, messaging, login, video/voice).

-----

*This changelog covers the full history of the BUTL Protocol from initial concept through the current release.*