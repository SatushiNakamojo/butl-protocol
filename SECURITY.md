# Security Policy

## BUTL Protocol v1.2

-----

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Security vulnerabilities must be reported privately so they can be assessed and addressed before being disclosed publicly. This protects every person and application using the protocol.

**Email:** SATUSHINAKAMOJO@PROTONMAIL.COM

Include the following in the report:

- A clear description of the vulnerability
- Steps to reproduce it
- The potential impact (what could an attacker do?)
- Which component is affected (specification, Python implementation, Rust implementation, MVP)
- A suggested fix, if available

### What to Expect

This project may be maintained by only one person. Security reports will be prioritized above all other work, but response times may vary.

- **Acknowledgment:** Within 72 hours of the report being received
- **Initial assessment:** Within 7 days
- **Fix or mitigation:** Timeline depends on severity and complexity, communicated during assessment
- **Public disclosure:** After the fix is published and users have had reasonable time to update

The reporter will be kept informed at each stage. Credit will be given to the reporter in the public disclosure unless anonymity is requested.

-----

## Scope

This security policy covers the following components:

|Component                      |Location                  |Description                        |
|-------------------------------|--------------------------|-----------------------------------|
|Protocol Specification         |`spec/`                   |The formal BUTL protocol definition|
|Python Reference Implementation|`python/butl_v12.py`      |Production-ready Python library    |
|Rust Reference Implementation  |`rust/butl_v12.rs.md`     |Production-ready Rust library      |
|Python MVP                     |`mvp/butl_mvp_v12.py`     |Zero-dependency proof of protocol  |
|Rust MVP                       |`mvp/butl_mvp_v12_rust.md`|Rust proof of protocol             |

Vulnerabilities in any of these components — whether in the protocol design, the cryptographic construction, or the implementation code — are in scope.

-----

## What Qualifies as a Security Vulnerability

### In Scope

- A flaw in the protocol design that allows an attacker to forge signatures, decrypt messages not intended for them, bypass the verification gate, replay messages outside the freshness window, or impersonate another BUTL identity
- An implementation bug that produces incorrect cryptographic output (wrong signatures, wrong shared secrets, wrong hashes)
- An implementation bug that leaks private key material through error messages, logs, timing, memory, or any other side channel
- A flaw in the verify-before-download mechanism that allows payload data to reach the device before the gate passes
- A weakness in the address chaining mechanism that allows an attacker to hijack or forge a chain
- Any bypass of the Proof of Satoshi check when it is enabled

### Out of Scope

- Vulnerabilities in third-party dependencies (report these to the dependency maintainers directly, but a heads-up to this project is appreciated)
- Vulnerabilities in the Bitcoin protocol or Bitcoin network itself
- Social engineering attacks (e.g., tricking a user into sharing their private key)
- Issues that require physical access to the user’s device
- Theoretical attacks requiring quantum computers (acknowledged as a known future risk, tracked in the roadmap)
- Denial-of-service attacks against the GitHub repository or project infrastructure

-----

## Known Limitations

The following are acknowledged limitations of the current implementations. They are documented here for transparency and are not considered undisclosed vulnerabilities.

### Pure-Python MVP Is Not Constant-Time

The MVP (`butl_mvp_v12.py`) implements secp256k1 elliptic curve math in pure Python using big-integer arithmetic. This implementation is **not constant-time** and is vulnerable to timing side-channel attacks. An attacker who can precisely measure the time taken by signing or ECDH operations could theoretically extract the private key.

**Mitigation:** The MVP is a proof of concept, not a production tool. Production deployments must use the reference implementation with the `ecdsa` library (which wraps constant-time operations) or, ideally, bindings to `libsecp256k1` (the C library used by Bitcoin Core).

### Stub Blockchain Does Not Query Real Bitcoin

Both reference implementations include a `StubBlockchain` / `BlockchainInterface` that returns hardcoded values. It does not connect to the Bitcoin network. This means:

- Block height freshness is not verified against the real chain
- Proof of Satoshi balance checks always pass
- Block hash verification is not performed

**Mitigation:** Production deployments must implement a real blockchain provider that connects to a Bitcoin full node, Electrum server, or API (e.g., mempool.space). The stub is clearly documented as a development placeholder.

### No Secure Key Storage

The reference implementations do not include encrypted key storage. Private keys exist in memory as raw bytes and are not encrypted at rest.

**Mitigation:** Production deployments should use OS keychains (macOS Keychain, Windows DPAPI, Linux secret-service), hardware security modules (HSMs), or encrypted key files with password-derived keys (Argon2id + AES-256-GCM).

### No Key Revocation

If a private key is compromised, there is currently no protocol-level mechanism to revoke the associated BUTL identity or address chain. An attacker with the private key can continue signing messages and extending the chain.

**Mitigation:** Notify contacts out-of-band to stop trusting the compromised ThreadID. A protocol-level revocation mechanism is planned for v2.0.

### ECDH and ECDSA on the Same Key

The reference implementations use the same secp256k1 key for both ECDSA signing and ECDH key agreement by default. While no known attack exploits this on secp256k1, it mixes two security games (EU-CMA for signatures, CDH for key exchange) on the same key, which some cryptographers consider non-ideal.

**Mitigation:** Use the `BUTL-EphemeralPubKey` option, which generates a separate key for ECDH. This is recommended as a best practice and documented in the specification.

### Balance Check Depends on External Query

When Proof of Satoshi is enabled, the balance check requires querying the Bitcoin blockchain. This introduces an external trust dependency — the query could be spoofed by a compromised API, subject to timing gaps, or unavailable.

**Mitigation:** Run a local Bitcoin full node for the strongest guarantee. Cache balance results for a limited time (recommended: 10 minutes). When maximum security is needed without external dependencies, disable Proof of Satoshi — the core protocol operates as pure math without it.

-----

## Cryptographic Foundation

The security of the BUTL protocol rests on three well-established assumptions:

|Assumption                                                    |Primitive               |Status                                           |
|--------------------------------------------------------------|------------------------|-------------------------------------------------|
|Elliptic Curve Discrete Logarithm Problem (ECDLP) on secp256k1|ECDSA, ECDH             |No known weaknesses. Secures $1T+ in Bitcoin.    |
|Collision resistance of SHA-256                               |Hashing, integrity      |No practical attacks after 25+ years of analysis.|
|Security of AES-256-GCM                                       |Authenticated encryption|NIST standard. Used by TLS 1.3, SSH, IPsec.      |

BUTL introduces no novel cryptographic constructions. A break in any of these primitives would affect not only BUTL but Bitcoin, TLS, and the majority of internet security infrastructure.

-----

## Supported Versions

|Version|Status    |Security Updates|
|-------|----------|----------------|
|v1.2.x |Current   |Yes             |
|v1.1.x |Superseded|No              |
|v1.0.x |Superseded|No              |

Only the latest version receives security updates. Users of earlier versions should upgrade.

-----

## Security-Related Design Decisions

For context on why certain security decisions were made:

|Decision                        |Rationale                                     |Document                                                    |
|--------------------------------|----------------------------------------------|------------------------------------------------------------|
|Verify-before-download gate     |Prevent malware/phishing at protocol level    |[Specification §7](spec/BUTL_Protocol_Specification_v1.2.md)|
|Proof of Satoshi optional       |Remove external trust dependency from core    |[PoS Benefits](docs/PROOF_OF_SATOSHI_BENEFITS.md)           |
|New address per message         |Forward secrecy + unlinkability               |[Specification §9](spec/BUTL_Protocol_Specification_v1.2.md)|
|AAD binding in AES-GCM          |Prevent payload redirection between recipients|[Specification §5](spec/BUTL_Protocol_Specification_v1.2.md)|
|Dual license (MIT OR Apache 2.0)|Apache 2.0 patent retaliation protects users  |[License Analysis](legal/DUAL_LICENSE_ANALYSIS.md)          |

-----

## Responsible Disclosure

This project follows responsible disclosure practices. Vulnerabilities are fixed before they are disclosed publicly. The timeline for disclosure is:

1. **Report received** — acknowledged within 72 hours
1. **Assessment** — severity and scope determined within 7 days
1. **Fix developed** — timeline communicated to reporter
1. **Fix published** — new version released
1. **Public disclosure** — after users have had reasonable time to update (minimum 14 days after fix publication)

If a vulnerability is already being actively exploited in the wild, the timeline may be compressed to protect users.

-----

## Thank You

Security researchers who responsibly disclose vulnerabilities are making the protocol safer for everyone. This project values and respects that work. Reporters will be credited publicly (unless anonymity is requested) and will have the gratitude of every person the protocol protects.

-----

*The protocol’s security rests on math. Keeping the implementations secure rests on the community. Thank you for helping.*