# BUTL Protocol

## Bitcoin Universal Trust Layer

*A cryptographic identity, encryption, and trust layer for the internet, anchored to the Bitcoin blockchain.*

-----

### The internet has no trust layer. BUTL adds one.

Every time you send an email, call an API, log into a website, or send a message, you’re trusting a middleman — a certificate authority, a password database, a corporate identity provider. These systems get hacked, go offline, censor users, and charge rent.

BUTL replaces them with math.

Your identity is a Bitcoin address. Authentication is an ECDSA signature. Encryption is ECDH + AES-256-GCM. Freshness is a Bitcoin block height. No signup. No company. No permission. Just the same cryptographic primitives that secure over $1 trillion in Bitcoin, repurposed as a universal trust layer for all internet communication.

BUTL is not a blockchain. It is not a token. It is a protocol — like TLS, but anchored to Bitcoin instead of certificate authorities.

-----

### Core Properties

|Property            |Mechanism                  |What It Means                                      |
|--------------------|---------------------------|---------------------------------------------------|
|**Identity**        |Bitcoin address (secp256k1)|You are your key. No signup. No third party.       |
|**Authentication**  |ECDSA signature            |Only you can sign as you. Unforgeable.             |
|**Encryption**      |ECDH + AES-256-GCM         |Only the intended recipient can read it.           |
|**Integrity**       |SHA-256 payload hash       |Any tampering is detected instantly.               |
|**Freshness**       |Bitcoin block height       |Old messages are rejected. Replay attacks fail.    |
|**Forward Privacy** |Address chaining           |New address every message. Unlinkable by observers.|
|**Device Safety**   |Verify-before-download     |Nothing touches your device until verified.        |
|**Sybil Resistance**|Proof of Satoshi (optional)|Economic cost to fake identities. Configurable.    |

-----

### How It Works

```
SENDER                                          RECEIVER
  │                                                │
  │  1. Generate fresh keypair (Bitcoin address)   │
  │  2. ECDH shared secret with receiver           │
  │  3. AES-256-GCM encrypt the message            │
  │  4. Build BUTL header                          │
  │  5. ECDSA sign the header                      │
  │                                                │
  │─── BUTL Header (cleartext) ───────────────────>│
  │                                                │
  │              ┌─── BUTL GATE ───────────────────┤
  │              │                                 │
  │              │  1. Structure check        ✓/✗  │
  │              │  2. Receiver match         ✓/✗  │
  │              │  3. Signature (ECDSA)      ✓/✗  │
  │              │  4. Freshness (block hgt)  ✓/✗  │
  │              │  5. Proof of Satoshi  [optional]│
  │              │  6. Chain proof            ✓/✗  │
  │              │                                 │
  │              │  ANY FAIL → connection dropped  │
  │              │  ALL PASS → gate opens          │
  │              └─────────────────────────────────┤
  │                                                │
  │─── Encrypted Payload ─────────────────────────>│
  │                                                │
  │              │  7. Payload hash (SHA-256) ✓/✗  │
  │              │  8. Decrypt (AES-GCM)      ✓/✗  │
  │                                                │
  │              ✓ VERIFIED — deliver to app       │
  │              ✗ FAIL — zero and discard         │
```

**No payload touches the receiver’s device until all header checks pass.** This is verify-before-download — the inverse of how most protocols work, and the reason BUTL blocks malware, phishing, and supply-chain attacks at the gate.

-----

### Architecture

```
┌─────────────────────────────────────────────────┐
│              APPLICATION LAYER                  │
│                                                 │
│   Email    Chat    API    Login    VoIP    IoT  │
├─────────────────────────────────────────────────┤
│              BUTL PROTOCOL                      │
│                                                 │
│   Identity · Encryption · Integrity             │
│   Freshness · Verify-Before-Download            │
│   [Optional: Proof of Satoshi]                  │
├─────────────────────────────────────────────────┤
│              BITCOIN NETWORK                    │
│                                                 │
│   SHA-256 · secp256k1 · ECDH · Blockchain       │
├─────────────────────────────────────────────────┤
│              TRANSPORT (TCP/UDP/QUIC)           │
└─────────────────────────────────────────────────┘
```

BUTL sits between the application and transport layers. It doesn’t replace TCP/IP or TLS — it adds a trust layer on top. Any application that can attach headers can use BUTL.

-----

### Proof of Satoshi: Optional by Design

The balance check is a receiver-side toggle with a configurable threshold slider.

**When disabled:** BUTL is 100% pure mathematics. Zero external dependencies. Every verification step is a local computation.

**When enabled:** The receiver sets the minimum satoshi requirement (1 sat — 21,000,000 BTC). Senders who don't meet the threshold are rejected at the gate. This creates economic cost to identity creation, preventing spam and bot farms.

The protocol provides the mechanism. The application chooses the policy.

-----

### Prove It Yourself

**One command. Eight proofs. Zero dependencies.**

```bash
python3 mvp/butl_mvp_v12.py
```

This runs a complete cryptographic proof of every BUTL claim — real Bitcoin addresses, real ECDSA signatures, real ECDH encryption, real address chaining, and the full 8-step verification gate — plus 4 Proof of Satoshi scenarios (disabled, enabled + passes, enabled + threshold too high, enabled + zero balance).

If you see `ALL 8 PROOFS PASSED`, the protocol works. On your machine. With your own eyes.

No internet connection. No accounts. No dependencies. Just Python 3 and the file.

See <mvp/HOW_TO_PROVE_BUTL_WORKS.md> for a step-by-step guide assuming zero coding experience.

-----

### Use the Reference Implementations

**Python** (866 lines, production-ready API):

```bash
pip install ecdsa pycryptodome
```

```python
from butl_v12 import BUTLKeypair, BUTLKeychain, BUTLSigner, BUTLGate, ProofOfSatoshiConfig

# Create your identity
me = BUTLKeypair()

# Create a keychain for sending (new address per message)
keychain = BUTLKeychain()
signer = BUTLSigner(keychain)

# Send an encrypted, signed message
msg = signer.sign_and_encrypt(b"Hello!", receiver.public_key, receiver.address)

# Receive and verify (two-phase gate)
gate = BUTLGate(my_keypair, pos_config=ProofOfSatoshiConfig(enabled=False))
report = gate.check_header(msg.header)          # Phase 1: header only
if report.gate_passed:
    plaintext, report = gate.accept_payload(     # Phase 2: decrypt
        msg.header, msg.encrypted_payload, report
    )
```

See <python/README.md> for the complete beginner’s guide.

**Rust** (Cargo.toml + full source):

See <rust/README.md> for setup and usage.

-----

### Repository Structure

```
butl-protocol/
│
├── LICENSE.md                         Dual license: MIT OR Apache 2.0
├── LICENSE-MIT.md                     MIT License
├── LICENSE-APACHE.md                  Apache License 2.0 (includes patent grant)
├── PATENTS.md                         Patent grant (applies to ALL users)
├── NOTICE                             Attribution and dependencies
├── README.md                          This file
├── CONTRIBUTING.md                    Contributor license agreement
├── CODE_OF_CONDUCT.md                 Community standards
├── SECURITY.md                        Vulnerability reporting
├── CHANGELOG.md                       Version history
├── ROADMAP.md                         Future plans
├── FAQ.md                             Frequently asked questions
├── GLOSSARY.md                        Term definitions
│
├── BUTL_WHITE_PAPER.md                White paper (Bitcoin paper style)
├── BUTL_ONE_PAGER.md                  60-second overview
├── BUTL_DIAGRAMS.md                   11 protocol diagrams
├── KEY_IS_IDENTITY_WARNING.md         Your key is your identity — protect it
├── WHY_ADDRESS_AND_PUBKEY.md          Why BUTL-ID has both
│
├── spec/                              Technical specification
│   ├── BUTL_Protocol_Specification_v1.2.md
│   ├── HEADER_REGISTRY.md
│   ├── VERSIONING.md
│   ├── WELL_KNOWN_BUTL_SPEC.md
│   └── TEST_VECTORS.md
│
├── python/                            Python implementation
│   ├── butl_v12.py
│   └── README.md
│
├── rust/                              Rust implementation
│   ├── butl_v12.rs
│   ├── Cargo.toml
│   └── README.md
│
├── mvp/                               Zero-dependency proof
│   ├── butl_mvp_v12.py
│   ├── butl_mvp_v12.rs
│   ├── Cargo.toml
│   ├── HOW_TO_PROVE_BUTL_WORKS.md
│   ├── VERIFICATION_CHECKLIST.md
│   └── QUICK_REFERENCE.md
│
├── docs/                              Documentation
│   ├── BUTL_PURE_MATH_PROOF.md
│   ├── CONFIDENCE_WITHOUT_POS.md
│   ├── PROOF_OF_SATOSHI_BENEFITS.md
│   ├── DEVELOPER_WARNING_POS_DISABLED.md
│   ├── BUTL_ECOSYSTEM_ARCHITECTURE.md
│   └── INTEGRATION_EXAMPLES.md
│
├── legal/                             Legal protections
│   ├── DEFENSIVE-PATENT-PLEDGE.md
│   └── GITHUB_GUIDE.md
│
└── ai/                                AI context transfer
    ├── AI_BRIEFING_DOCUMENT.md
    ├── AI_QUICK_START_PROMPT.md
    └── AI_CODE_GENERATION_REFERENCE.md
```

-----

### The Math

BUTL uses three cryptographic primitives:

|Primitive                    |Specification  |Secures                             |
|-----------------------------|---------------|------------------------------------|
|**SHA-256**                  |FIPS 180-4     |Bitcoin mining, TLS, SSH, Git, PGP  |
|**ECDSA / ECDH on secp256k1**|SEC 2          |Bitcoin ($1T+), Ethereum            |
|**AES-256-GCM**              |NIST SP 800-38D|TLS 1.3, IPsec, SSH, every major VPN|

No new cryptography. No new assumptions. The same primitives that secure the internet and the Bitcoin network, composed into a trust layer.

Without Proof of Satoshi, every verification step is a local computation — **100% pure mathematics** with zero external trust dependencies. See <docs/BUTL_PURE_MATH_PROOF.md>.

-----

### Use Cases

**Tier 1 — Transformative:** Passwordless login, email anti-phishing, encrypted messaging, API authentication.

**Tier 2 — High value:** Financial verification, healthcare records, IoT authentication, document signing, supply chain.

**Tier 3 — Significant:** Social media identity, voting, VPN auth, whistleblowing, gaming identity.

**Tier 4 — Niche:** Academic credentials, DNS signing, firmware updates, content authenticity.

**Advanced:** BUTL at the boot level — firmware and OS verified before execution.

See <docs/INTEGRATION_EXAMPLES.md> for code snippets covering login, email, API, IoT, and file transfer.

-----

### Roadmap

|Phase|Target |Focus                                                                         |
|-----|-------|------------------------------------------------------------------------------|
|v1.2 |Now    |Specification, implementations, MVP, documentation                            |
|v1.3 |Q3 2026|Production libraries (PyPI, crates.io, npm), formal test suite, key revocation|
|v2.0 |2027   |Multi-party signatures, post-quantum migration path, IETF RFC submission      |
|v3.0+|2028+  |Hardware integration, streaming mode, mobile SDKs                             |

See <ROADMAP.md> for full details.

-----

### Contributing

Contributions are welcome. By contributing, you agree to the dual license (MIT OR Apache 2.0), the PATENTS grant, and the Defensive Patent Pledge.

See <CONTRIBUTING.md> for guidelines.

-----

### License

Dual licensed under **MIT** or **Apache 2.0**, at your option.

The [PATENTS](PATENTS.md) grant and [Defensive Patent Pledge](legal/DEFENSIVE-PATENT-PLEDGE.md) apply to all users regardless of license choice.

All methods described in this repository are published as prior art and cannot be patented.

-----

### Links

|Document                                                 |Description                                    |
|---------------------------------------------------------|-----------------------------------------------|
|[White Paper](BUTL_WHITE_PAPER.md)                       |Full protocol description (Bitcoin paper style)|
|[One-Pager](BUTL_ONE_PAGER.md)                           |60-second overview                             |
|[Specification](spec/BUTL_Protocol_Specification_v1.2.md)|Complete technical spec                        |
|[Prove It Works](mvp/HOW_TO_PROVE_BUTL_WORKS.md)         |Step-by-step for beginners                     |
|[FAQ](FAQ.md)                                            |20 common questions answered                   |
|[Key Safety](KEY_IS_IDENTITY_WARNING.md)                 |Your key is your identity — read this          |

-----

*BUTL — Because the internet deserves a trust layer that doesn’t trust anyone.*