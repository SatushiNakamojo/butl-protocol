# Frequently Asked Questions

## BUTL Protocol

-----

## What Is BUTL?

### What does BUTL stand for?

Bitcoin Universal Trust Layer.

### What is the BUTL Protocol?

BUTL is an open cryptographic protocol that adds identity verification, end-to-end encryption, message integrity, replay protection, forward privacy, and optional sybil resistance to any internet communication. It uses Bitcoin’s existing cryptographic infrastructure — SHA-256, secp256k1 ECDSA/ECDH, block heights, and optionally UTXO balances — as a universal trust anchor.

### Is BUTL a cryptocurrency?

No. BUTL has no token, no coin, and no blockchain of its own. It is a communication protocol — like HTTP or TLS — that uses Bitcoin’s existing cryptography as a trust anchor.

### Is BUTL a blockchain?

No. BUTL does not create, maintain, or require its own blockchain. It reads data from the Bitcoin blockchain (block heights and optionally address balances) but never writes to it.

### What problem does BUTL solve?

The internet has no native trust layer. Every existing trust mechanism — passwords, certificate authorities, OAuth, API keys — depends on a centralized intermediary that can be hacked, go offline, censor users, or charge rent. BUTL replaces them with cryptographic proof that depends on math instead of middlemen.

### How is BUTL different from TLS?

TLS depends on certificate authorities — centralized organizations that can be compromised, coerced, or revoked. BUTL depends on Bitcoin addresses — self-sovereign cryptographic identities that no one can create, revoke, or control except the key holder. TLS provides transport-layer encryption. BUTL provides application-layer identity, encryption, integrity, freshness, sybil resistance, and verify-before-download.

### How is BUTL different from PGP?

PGP provides signatures and encryption but has no freshness mechanism (no replay protection), no sybil resistance, no verify-before-download, no address chaining (no automatic key rotation), and notoriously difficult key management. BUTL addresses all of these.

### How is BUTL different from the Signal Protocol?

The Signal Protocol provides excellent encryption but depends on Signal’s key distribution servers. BUTL’s key discovery uses `.well-known/butl.json` or DNS TXT records — no single company in the path. Signal is a messaging protocol. BUTL is a universal trust layer for any application.

### How is BUTL different from OAuth?

OAuth depends on an identity provider (Google, Facebook, Apple) who can revoke access at will. BUTL identity is a Bitcoin address controlled by the holder. No one can revoke, suspend, or delete a BUTL identity.

-----

## How It Works

### What is a BUTL identity?

A Bitcoin address derived from a secp256k1 keypair. The holder generates a private key, derives a public key and Bitcoin address, and possesses their identity — without registering with any authority. The address is the identity. The public key enables cryptographic operations. The private key proves ownership.

### Why does a BUTL-ID have both an address and a public key?

The address is for identification — it’s short, human-readable, has error detection, and is what the Bitcoin blockchain is indexed by. The public key is for cryptography — it’s what signature verification, ECDH encryption, and chain proof verification actually use. The address is derived from the public key through a one-way hash, so the public key cannot be recovered from the address alone. Both must be present. See <WHY_ADDRESS_AND_PUBKEY.md> for a detailed explanation.

### What is the verify-before-download gate?

The core security mechanism of BUTL. Messages are delivered in two phases. Phase 1: only the header is transmitted, and the receiver runs 6 verification checks (structure, receiver match, signature, freshness, optional balance, chain proof). If any check fails, the connection is dropped — no payload is ever transmitted or saved. Phase 2: only if all checks pass, the encrypted payload is downloaded, hash-verified, and decrypted. Nothing touches the device until it’s verified.

### What is address chaining?

Each BUTL message uses a fresh Bitcoin address. To prove that the same person sent multiple messages, the old private key signs the new address (a “chain proof”). This provides forward secrecy (compromising one key affects one message), privacy (observers see unrelated addresses), and verifiable continuity (the receiver can confirm it’s the same sender).

### What is a ThreadID?

The SHA-256 hash of the first sender address in an address chain. It uniquely identifies a conversation thread and remains constant across all messages in the chain, even as the sender’s address changes with every message.

### What cryptography does BUTL use?

Three primitives: SHA-256 (hashing and integrity), ECDSA on secp256k1 (digital signatures), and ECDH + AES-256-GCM (encryption). These are the same primitives used by Bitcoin, TLS 1.3, and SSH. BUTL introduces no novel cryptographic constructions.

### Does BUTL require changes to Bitcoin?

No. BUTL uses Bitcoin as-is. It reads block heights and optionally address balances. It does not write to the blockchain or require any Bitcoin protocol changes.

-----

## Proof of Satoshi

### What is Proof of Satoshi?

An optional feature where the receiver checks that the sender’s Bitcoin address holds a minimum number of satoshis. It creates an economic cost to creating identities, preventing spam and bot farms.

### Why is Proof of Satoshi optional?

Because it is the only part of BUTL that requires querying the Bitcoin blockchain — an external dependency. Making it optional means the core protocol is 100% pure math with zero external trust dependencies. Applications that need sybil resistance can turn it on. Applications that don’t can leave it off and operate with 99.7% confidence based on math alone.

### Who controls the Proof of Satoshi threshold?

The receiver. The sender has no say — the receiver decides how much economic stake is required. This can range from 1 satoshi to 21,000,000 BTC (the total Bitcoin supply), configured via a slider. Implementations should use a logarithmic scale.

### Does BUTL require owning Bitcoin?

No. With Proof of Satoshi disabled (the default), only a secp256k1 keypair is needed. No wallet, no exchange account, no satoshis. If Proof of Satoshi is enabled by the receiver, then the sender needs a funded Bitcoin address.

### What happens if Proof of Satoshi is disabled?

The protocol still provides identity, authentication, encryption, integrity, freshness, address chaining, and verify-before-download. The only missing property is sybil resistance — unlimited identities can be created at zero cost. Applications that need sybil resistance should enable Proof of Satoshi or implement alternative mechanisms. See [DEVELOPER_WARNING_POS_DISABLED.md](docs/DEVELOPER_WARNING_POS_DISABLED.md).

-----

## Security

### How secure is BUTL?

Without Proof of Satoshi: 99.7% confidence. Every verification step is a local computation using the same primitives that secure Bitcoin ($1T+), TLS, and SSH. The 0.3% residual accounts for the universal risk that any software implementation may contain bugs.

With Proof of Satoshi: 97% confidence. The balance check introduces an external trust dependency (the blockchain query) that could theoretically be spoofed.

See [CONFIDENCE_WITHOUT_POS.md](docs/CONFIDENCE_WITHOUT_POS.md) for the full analysis.

### What would break BUTL?

A break in the secp256k1 elliptic curve discrete logarithm problem. This would also break Bitcoin, Ethereum, and most public-key cryptography on the internet. It is not a BUTL-specific risk — it is a civilizational risk that the entire cryptographic community actively monitors.

### Is BUTL post-quantum secure?

No. Like Bitcoin and TLS, BUTL uses elliptic curve cryptography that is theoretically vulnerable to Shor’s algorithm on a fault-tolerant quantum computer. Such computers do not exist and are not expected for 10+ years. When they arrive, BUTL will migrate to post-quantum algorithms (hybrid ECDSA + Dilithium, hybrid ECDH + ML-KEM) alongside the rest of the internet. This is tracked in the <ROADMAP.md>.

### What happens if the private key is lost?

The identity is gone. Permanently. There is no recovery mechanism, no reset email, no customer support. This is the fundamental tradeoff of self-sovereign identity: no one can take it from you, and no one can recover it for you. See <KEY_IS_IDENTITY_WARNING.md> for backup instructions and protection strategies.

### What happens if the private key is compromised?

An attacker with the private key can sign messages as that identity and continue the address chain. There is currently no protocol-level revocation mechanism (planned for v2.0). Contacts must be notified out-of-band to stop trusting the compromised ThreadID.

### Can BUTL work offline?

With Proof of Satoshi disabled and a cached block height, all verification is local math. Full offline operation is possible if the receiver accepts a wider freshness window. The block height is a single integer that changes approximately every 10 minutes and can be cached for extended periods.

-----

## Using BUTL

### How do I prove BUTL works?

One command:

```bash
python3 mvp/butl_mvp_v12.py
```

This runs 8 cryptographic proofs and 4 Proof of Satoshi scenarios with zero dependencies. If it says “ALL 8 PROOFS PASSED,” the protocol works. On your machine. See [HOW_TO_PROVE_BUTL_WORKS.md](mvp/HOW_TO_PROVE_BUTL_WORKS.md) for a step-by-step guide assuming zero coding experience.

### What programming languages are supported?

Python and Rust have full reference implementations. The protocol is language-agnostic — any language with SHA-256, secp256k1, and AES-256-GCM support can implement BUTL. JavaScript, Go, and C implementations are planned for v1.3.

### What applications can use BUTL?

Any application that communicates between two or more parties: email, messaging, API authentication, website login, video/voice calls, IoT device communication, document signing, file transfer, and more. BUTL is transport-agnostic — it works over TCP, UDP, QUIC, HTTP, WebSocket, SMTP, MQTT, Bluetooth, or any other mechanism that can move bytes. See [USE_CASES.md](docs/USE_CASES.md) for 20 ranked use cases.

### What is `.well-known/butl.json`?

A standard endpoint for publishing BUTL receiver public keys. A server at `https://example.com/.well-known/butl.json` returns a JSON object containing the receiver’s Bitcoin address, public key, and configuration preferences (Proof of Satoshi requirements, freshness window, supported versions). This enables sender-side key discovery without any centralized directory. See [WELL_KNOWN_BUTL_SPEC.md](spec/WELL_KNOWN_BUTL_SPEC.md).

-----

## Legal

### What license is BUTL under?

Dual licensed under **MIT** or **Apache 2.0**, at the user’s choice. This follows the same pattern used by Rust, serde, and the broader Rust ecosystem. See <LICENSE.md> for details on which to choose.

### Can someone patent BUTL?

No. All methods are published as prior art under the dual license. The <PATENTS.md> file provides an explicit, irrevocable patent grant covering all BUTL methods. The [Defensive Patent Pledge](legal/DEFENSIVE-PATENT-PLEDGE.md) commits all contributors to never file patents on BUTL. Anyone who attempts to patent BUTL’s methods after the publication date will face invalidation based on the timestamped prior art in this repository.

### Which license should I choose?

**Apache 2.0** if patent protection matters. The patent grant and retaliation clause are built into the license itself (Section 3). Recommended for commercial products and enterprise use.

**MIT** if simplicity and pre-approval matter. Some corporate legal teams have MIT pre-approved. The separate PATENTS file and Defensive Patent Pledge still provide patent protection.

Either way, commercial use is permitted. Neither license requires open-sourcing your own code.

### Can I use BUTL in a commercial product?

Yes. Both MIT and Apache 2.0 allow unrestricted commercial use, modification, and distribution.

### Can I fork BUTL?

Yes. Both licenses permit forks. A fork can be renamed, modified, and distributed independently.

-----

## About the Project

### Who created BUTL?

BUTL is an open protocol created by its contributors and released to the public. It belongs to no company or individual. See <AI_COLLABORATION_DISCLOSURE.md> for transparency on how the protocol was developed.

### Was BUTL generated by AI?

Yes. All documents, code, and specifications in v1.2 were initially generated by Claude (Anthropic), with additional collaboration from Grok (xAI) and ChatGPT (OpenAI). The protocol was conceived and directed by its human creator. AI systems served as collaborative tools. All materials may have been edited since generation. See <AI_COLLABORATION_DISCLOSURE.md> for full details.

### How is this project maintained?

This project may be maintained by only one person. Response times on issues and pull requests may vary. See <CONTRIBUTING.md> for how to contribute and what to expect.

### How can I contribute?

Report bugs, suggest features, improve documentation, write code, or propose protocol changes. See <CONTRIBUTING.md> for the full guide.

### Where can I learn more?

|Want to…                             |Read this                                 |
|-------------------------------------|------------------------------------------|
|Understand the protocol in 60 seconds|<BUTL_ONE_PAGER.md>                       |
|Read the full protocol description   |<BUTL_WHITE_PAPER.md>                     |
|See the technical specification      |<spec/BUTL_Protocol_Specification_v1.2.md>|
|Prove it works on your computer      |<mvp/HOW_TO_PROVE_BUTL_WORKS.md>          |
|Build with Python                    |<python/README.md>                        |
|Build with Rust                      |<rust/README.md>                          |
|Understand key safety                |<KEY_IS_IDENTITY_WARNING.md>              |
|See the development roadmap          |<ROADMAP.md>                              |
|Look up a term                       |<GLOSSARY.md>                             |

-----

*Question not answered here? Open a GitHub issue with the label `question`.*