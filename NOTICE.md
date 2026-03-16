# NOTICE

## BUTL Protocol — Bitcoin Universal Trust Layer

**Copyright 2026 Satushi Nakamojo**

-----

### About This Project

BUTL (Bitcoin Universal Trust Layer) is an open cryptographic protocol that
provides identity verification, end-to-end encryption, message integrity,
replay protection, forward privacy, and optional sybil resistance for any
internet communication. It uses Bitcoin’s existing cryptographic infrastructure
as a universal trust anchor.

BUTL is not a blockchain. It is not a token. It is a protocol — open,
permanent, and free for all to use.

-----

### Licensing

This project is dual-licensed under either:

- **MIT License** — see <LICENSE-MIT.md>
- **Apache License, Version 2.0** — see <LICENSE-APACHE.md>

at your option.

The <PATENTS.md> grant and
[DEFENSIVE-PATENT-PLEDGE.md](legal/DEFENSIVE-PATENT-PLEDGE.md) apply to all
users regardless of license choice.

-----

### Cryptographic Primitives

This software uses the following well-established cryptographic standards:

|Primitive       |Specification                |Usage in BUTL                                      |
|----------------|-----------------------------|---------------------------------------------------|
|SHA-256         |FIPS 180-4 (NIST, 2015)      |Message hashing, payload integrity, thread identity|
|ECDSA           |SEC 2 (SECG, 2010), secp256k1|Digital signatures and verification                |
|ECDH            |SEC 1 (SECG, 2009), secp256k1|Shared secret derivation for encryption            |
|AES-256-GCM     |NIST SP 800-38D (2007)       |Authenticated payload encryption                   |
|RIPEMD-160      |ISO/IEC 10118-3 (1996)       |Bitcoin address derivation                         |
|Base58Check     |Bitcoin Protocol             |Legacy address encoding                            |
|Bech32 / Bech32m|BIP 173 / BIP 350            |SegWit address encoding                            |

No novel cryptographic constructions are introduced. BUTL composes existing,
battle-tested primitives.

-----

### Standards Referenced

|Standard|Issuer |Referenced In                               |
|--------|-------|--------------------------------------------|
|BIP-32  |Bitcoin|Hierarchical deterministic key derivation   |
|BIP-39  |Bitcoin|Mnemonic seed phrases for identity backup   |
|BIP-173 |Bitcoin|Bech32 address encoding                     |
|BIP-350 |Bitcoin|Bech32m address encoding                    |
|RFC 6979|IETF   |Deterministic ECDSA signatures              |
|RFC 8615|IETF   |.well-known URI convention for key discovery|

-----

### Third-Party Dependencies

#### Python Reference Implementation

|Package                 |License     |Usage                                                |
|------------------------|------------|-----------------------------------------------------|
|`ecdsa`                 |MIT         |secp256k1 keypair, ECDSA signing, ECDH key agreement |
|`pycryptodome`          |BSD 2-Clause|AES-256-GCM authenticated encryption                 |
|`argon2-cffi` (optional)|MIT         |Password-based key derivation for identity encryption|
|`mnemonic` (optional)   |MIT         |BIP-39 seed phrase generation                        |
|`qrcode` (optional)     |BSD         |QR code display for identity sharing                 |

#### Rust Reference Implementation

|Crate           |License          |Usage                                 |
|----------------|-----------------|--------------------------------------|
|`secp256k1` 0.29|CC0-1.0          |secp256k1 keypair, ECDSA, ECDH        |
|`sha2` 0.10     |MIT OR Apache 2.0|SHA-256 hashing                       |
|`ripemd` 0.1    |MIT OR Apache 2.0|RIPEMD-160 for address derivation     |
|`aes-gcm` 0.10  |MIT OR Apache 2.0|AES-256-GCM encryption                |
|`base64` 0.22   |MIT OR Apache 2.0|Base64 encoding for signatures        |
|`rand` 0.8      |MIT OR Apache 2.0|Cryptographic random number generation|
|`serde` 1.0     |MIT OR Apache 2.0|Serialization                         |
|`serde_json` 1.0|MIT OR Apache 2.0|JSON serialization                    |
|`bs58` 0.5      |MIT OR Apache 2.0|Base58Check encoding                  |
|`hex` 0.4       |MIT OR Apache 2.0|Hexadecimal encoding                  |

#### Minimum Viable Prototype

The Python MVP (`butl_mvp_v12.py`) has **zero external dependencies**. It
implements secp256k1 elliptic curve math, ECDSA, ECDH, and authenticated
encryption entirely in pure Python using only the standard library.

-----

### Acknowledgments

The BUTL protocol builds upon the foundational work of:

- **Satoshi Nakamoto** — Bitcoin: A Peer-to-Peer Electronic Cash System (2008).
  BUTL uses Bitcoin’s cryptographic infrastructure (SHA-256, secp256k1, block
  heights, UTXO model) as its trust anchor.
- **Neal Koblitz and Victor Miller** — Independent co-inventors of elliptic
  curve cryptography (1985). The secp256k1 curve underlies all BUTL identity,
  signing, and encryption operations.
- **Joan Daemen and Vincent Rijmen** — Inventors of Rijndael, selected as the
  Advanced Encryption Standard (AES) by NIST (2001). AES-256-GCM provides
  BUTL’s authenticated payload encryption.
- **The Bitcoin developer community** — For establishing and maintaining the
  secp256k1, Base58Check, Bech32, BIP-32, and BIP-39 standards that BUTL
  builds upon.
- **The IETF and NIST** — For publishing and maintaining the cryptographic
  standards (SHA-256, AES-GCM, RFC 6979, RFC 8615) that BUTL relies on.

-----

### Prior Art Declaration

All methods, algorithms, processes, data structures, and techniques described
in this repository were first publicly disclosed on the date of the initial
commit. This publication constitutes prior art under 35 U.S.C. Section 102
(United States), Article 54 EPC (European Patent Convention), and equivalent
provisions in all jurisdictions.

-----

### Contact

- **Repository:** github.com/satushinakamojo/butl-protocol (https://github.com/satushinakamojo/butl-protocol)
- **Security issues:** See <SECURITY.md>
- **General inquiries:** See <FAQ.md>

-----

*BUTL — Bitcoin Universal Trust Layer*
*Open protocol. Dual licensed. Free forever.*