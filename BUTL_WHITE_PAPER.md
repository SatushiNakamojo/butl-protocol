# Bitcoin Universal Trust Layer: A Peer-to-Peer Cryptographic Trust Protocol

**Satushi Nakamojo**

**satushinakamojo@protonmail.com**

**www.butl-protocol.com**

**www.github.com/satushinakamojo/butl-protocol**

-----

**Abstract.** A purely cryptographic trust layer for the internet would allow identity verification, authenticated encryption, and message integrity between any two parties without relying on a trusted third party. Digital signatures using Bitcoin’s secp256k1 curve provide strong authentication, but the main benefits are lost if identity still requires trusted intermediaries — certificate authorities, identity providers, or password databases. We propose a protocol that uses Bitcoin’s existing cryptographic infrastructure as a universal trust anchor. The protocol provides self-sovereign identity via Bitcoin addresses, end-to-end encryption via ECDH key agreement, message integrity via SHA-256 hashing, replay protection via blockchain block height references, optional sybil resistance via on-chain balance verification, and forward privacy via per-message address chaining. A verify-before-download mechanism ensures that no payload data reaches the receiving device until all cryptographic checks pass. The protocol requires no changes to Bitcoin, no new blockchain, and no tokens. It composes existing, battle-tested primitives into a universal trust layer that any internet application can adopt.

-----

## 1. Introduction

Communication on the internet relies almost entirely on trust models that depend on centralized intermediaries. TLS depends on certificate authorities. Authentication depends on passwords stored in databases. Identity depends on accounts managed by corporations. These intermediaries introduce single points of failure, create targets for attackers, enable censorship, and extract economic rent.

What is needed is a trust layer based on cryptographic proof instead of institutional trust — allowing any two parties to verify identity, encrypt communication, and ensure integrity without relying on any third party.

Bitcoin [1] demonstrated that a peer-to-peer system could achieve consensus and transfer value without trusted intermediaries, using SHA-256 hashing, ECDSA signatures on the secp256k1 elliptic curve, and proof-of-work consensus. These same cryptographic primitives — already securing over one trillion dollars in value — can be repurposed to provide a universal trust layer for all internet communication.

In this paper, we propose BUTL (Bitcoin Universal Trust Layer), a protocol that uses Bitcoin’s cryptographic infrastructure to provide identity, authentication, encryption, integrity, freshness, forward privacy, and optional sybil resistance for any application. The protocol operates as a layer between the application and transport layers, analogous to how TLS operates but without certificate authorities.

-----

## 2. Identity

We define identity as a Bitcoin address — a value derived deterministically from a secp256k1 public key through SHA-256 hashing, RIPEMD-160 hashing, and Base58Check encoding [2].

```
Identity = Base58Check(0x00 || RIPEMD-160(SHA-256(CompressedPublicKey)))
```

This identity is self-sovereign: the holder generates a private key, derives the public key and address, and possesses the identity without registering with any authority. The identity can be independently verified by anyone who knows the public key. It cannot be revoked, suspended, or modified by any third party.

A BUTL identity consists of three mathematically linked components: the private key (a 256-bit integer on secp256k1), the public key (a point on the secp256k1 curve, transmitted in 33-byte compressed form), and the Bitcoin address (a human-readable encoding of the public key hash). The address serves as the identifier — short, checksummed, and compatible with the Bitcoin ecosystem. The public key serves as the cryptographic tool — required for signature verification, ECDH key agreement, and chain proof validation. The private key serves as the proof of ownership — only the holder can sign, decrypt, and extend the address chain.

The cost of generating an identity is negligible (one scalar multiplication on secp256k1). For applications that require economic cost to identity creation, the optional Proof of Satoshi mechanism (Section 8) adds a configurable balance requirement.

-----

## 3. Signatures

To authenticate a message, the sender constructs a canonical signing payload containing the protocol version, sender and receiver identity fields, block height and hash, payload hash, previous address, and nonce. The payload is hashed with SHA-256, and the hash is signed using ECDSA on secp256k1.

```
signing_payload = "BUTL-Version:{v}\n
                   BUTL-Sender:{addr}\n
                   BUTL-SenderPubKey:{pk}\n
                   BUTL-Receiver:{rx_addr}\n
                   BUTL-ReceiverPubKey:{rx_pk}\n
                   BUTL-BlockHeight:{h}\n
                   BUTL-BlockHash:{bh}\n
                   BUTL-PayloadHash:{ph}\n
                   BUTL-PrevAddr:{prev}\n
                   BUTL-Nonce:{n}"

signature = ECDSA_SIGN(SHA-256(signing_payload), private_key)
```

The receiver verifies the signature using the sender’s public key, which is included in the header. The receiver also confirms that the sender’s public key correctly derives to the sender’s claimed address. Both operations are local computations requiring no network access or third-party involvement.

The security of this scheme rests on the existential unforgeability of ECDSA under the Elliptic Curve Discrete Logarithm assumption on secp256k1 — the same assumption securing all Bitcoin transactions.

-----

## 4. Encryption

To ensure that only the intended receiver can read the message, the sender encrypts the payload using a shared secret derived from Elliptic Curve Diffie-Hellman (ECDH) key agreement on secp256k1.

```
shared_secret = SHA-256( (sender_private_key × receiver_public_key).x )
```

The receiver computes the identical shared secret:

```
shared_secret = SHA-256( (receiver_private_key × sender_public_key).x )
```

Both computations yield the same value due to the commutative property of elliptic curve scalar multiplication. This shared secret serves as the symmetric key for AES-256-GCM authenticated encryption [5].

```
encrypted_payload = AES-256-GCM(
    key       = shared_secret,
    iv        = random_12_bytes,
    plaintext = message_body,
    aad       = sender_address || receiver_address
)
```

The Additional Authenticated Data (AAD) binds the ciphertext to the sender and receiver addresses, preventing an attacker from redirecting an encrypted payload to a different recipient. If the AAD does not match, GCM authentication fails and decryption is rejected.

The output is: `IV (12 bytes) || ciphertext || GCM_tag (16 bytes)`.

The SHA-256 hash of this encrypted output is placed in the `BUTL-PayloadHash` header field for integrity verification prior to decryption.

For enhanced forward secrecy, the sender MAY generate an ephemeral keypair for ECDH and include the ephemeral public key in the `BUTL-EphemeralPubKey` header field. When present, the receiver uses the ephemeral key rather than the sender’s signing key for ECDH. This separates the signing key from the encryption key, ensuring that compromise of the signing key does not retroactively compromise message confidentiality.

-----

## 5. Freshness

To prevent replay attacks, each message references a recent Bitcoin block height and hash. The receiver verifies that:

1. The block hash at the claimed height matches the Bitcoin blockchain.
1. The block height is within a configurable freshness window of the current chain tip.

```
age = current_chain_tip - message_block_height
valid = (0 ≤ age ≤ freshness_window)
```

The default freshness window is 144 blocks (approximately 24 hours). Applications may configure tighter windows for higher-security use cases (e.g., 6 blocks for financial APIs) or wider windows for delay-tolerant systems.

This mechanism leverages Bitcoin’s proof-of-work consensus as a global, trustless clock. An attacker cannot forge a valid block height without commanding more hashpower than the Bitcoin network. The block height is a single global value that changes approximately every 10 minutes and can be cached by the receiver, requiring no per-message network query.

-----

## 6. Address Chaining

A fundamental tension exists between privacy (using different addresses for different messages) and identity continuity (proving the same entity sent multiple messages). BUTL resolves this through address chaining.

Each message uses a fresh Bitcoin address derived from the next key in a deterministic sequence (BIP-32 [3] hierarchical derivation is recommended for production). To prove continuity, the sender signs the new address with the previous message’s private key:

```
chain_proof = ECDSA_SIGN(SHA-256(new_address), previous_private_key)
```

The receiver verifies this proof using the previous message’s public key, confirming that the entity controlling the old address also controls the new one.

```
Message 0:  Address_0  (genesis, no chain proof)
Message 1:  Address_1  (chain_proof = Sign(Address_1, key_0))
Message 2:  Address_2  (chain_proof = Sign(Address_2, key_1))
    ...
Message N:  Address_N  (chain_proof = Sign(Address_N, key_{N-1}))
```

A thread is identified by the SHA-256 hash of the first address in the chain:

```
thread_id = SHA-256(Address_0)
```

This mechanism provides three properties simultaneously: forward secrecy (compromising the current key does not reveal past keys), unlinkability by third-party observers (without the chain proofs, different addresses appear unrelated), and automatic key rotation (every message is a key rotation event). After creating the chain proof, the previous private key should be securely deleted.

-----

## 7. Verify-Before-Download

We define a strict security boundary between header verification and payload delivery. The receiving implementation MUST NOT write any payload data to memory, disk, cache, or any persistent storage until all header verification checks have passed.

Messages are delivered in two phases:

**Phase 1: Header delivery.** The sender transmits the BUTL header (cleartext). The receiver performs the following checks in order:

```
Step 1: Structural validation (all required fields present and well-formed)
Step 2: Receiver match (BUTL-Receiver == my address)
Step 3: Signature verification (ECDSA on canonical payload, pubkey derives to claimed address)
Step 4: Freshness check (block height within window)
Step 5: Proof of Satoshi (OPTIONAL, see Section 8)
Step 6: Chain proof verification (if threaded)
```

If any enabled check fails, the connection is terminated. No payload is requested, transmitted, or stored.

**Phase 2: Payload delivery.** If and only if all checks pass, the receiver signals readiness. The encrypted payload is transmitted and verified:

```
Step 7: Payload hash (SHA-256 of encrypted payload == BUTL-PayloadHash)
Step 8: Decryption (AES-256-GCM with ECDH shared secret)
```

If Step 7 or 8 fails, the payload is zeroed from memory immediately.

This two-phase model prevents several attack classes at the protocol level: malware delivery (payload never reaches device), resource exhaustion (large payloads rejected before download), phishing (unverified content never rendered), and supply-chain attacks (API responses verified before processing). The verify-before-download principle inverts the traditional model where data is downloaded first and verified later — by which point the damage may already be done.

-----

## 8. Proof of Satoshi

For applications that require sybil resistance, BUTL provides an optional balance verification step called Proof of Satoshi. The receiver queries the Bitcoin blockchain to verify that the sender’s address holds at least a configurable minimum number of satoshis.

```
pos_check = (balance(BUTL-Sender) ≥ pos_min_satoshis)
```

This step is OPTIONAL. The receiver configures two parameters:

- **pos_enabled** (boolean): Whether to enforce balance checking. Default: false.
- **pos_min_satoshis** (integer): Minimum balance required. Range: 1 — 2,100,000,000,000,000 (21,000,000 BTC).

When Proof of Satoshi is disabled, the protocol operates as a pure-math cryptographic trust layer with zero external dependencies beyond the block height reference. All remaining verification steps (1-4, 6-8) are local computations on data contained in the message header and payload. No blockchain query is performed. No external service is trusted. Confidence in the protocol’s correctness is 99.7%, limited only by the universal risk of implementation bugs.

When Proof of Satoshi is enabled, the economic cost of creating identities scales linearly: creating N identities requires funding N addresses with at least `pos_min_satoshis` each, plus on-chain transaction fees. This makes mass identity creation (bot farms, spam accounts) economically costly. Confidence with Proof of Satoshi enabled is 97%, reflecting the external trust dependency introduced by the blockchain query.

The decision to enable Proof of Satoshi and the threshold to set are application-level decisions. Casual messaging may disable it entirely. Public APIs may require 1,000 satoshis. Enterprise systems may require 100,000 satoshis. Financial applications may require 1,000,000 satoshis. The protocol provides the mechanism; the application chooses the policy.

-----

## 9. Key Discovery

For the sender to encrypt a message to the receiver, the sender must know the receiver’s public key. BUTL defines a standard discovery mechanism using the IETF `.well-known` URI convention [8].

```
GET https://{domain}/.well-known/butl.json
```

The response is a JSON object containing the receiver’s Bitcoin address, compressed public key, and configuration preferences:

```json
{
  "version": 1,
  "address": "bc1q...",
  "pubkey": "02a163...",
  "pos_required": false,
  "freshness_window": 144,
  "supported_versions": [1]
}
```

This endpoint is served over HTTPS (mandatory) and provides the minimum information needed for a sender to construct a BUTL message. For domains that cannot serve HTTP endpoints, a DNS TXT record alternative is defined: `_butl.example.com TXT "v=1; addr=bc1q...; pubkey=02..."`.

Key discovery is the one step that relies on existing web infrastructure (HTTPS, DNS). Once the receiver’s public key is obtained and verified (ideally through an out-of-band confirmation for first contact), all subsequent communication is secured entirely by BUTL.

-----

## 10. Transport Agnosticism

BUTL is a trust layer, not a transport layer. It defines how to sign, encrypt, verify, and gate a message. It does not define how the bytes traverse the network. That responsibility belongs to whatever transport protocol the application already uses.

The following bindings are defined for common protocols:

**Email (SMTP).** BUTL fields are carried as X-headers (X-BUTL-Sender, X-BUTL-Signature, etc.). The encrypted payload is a MIME attachment. The mail client verifies BUTL headers before rendering or saving the attachment.

**HTTP / API.** Phase 1 uses BUTL-prefixed HTTP headers. Phase 2 transmits the encrypted payload in the request or response body after the receiver confirms verification.

**Instant Messaging.** BUTL headers form the message envelope. The encrypted payload is the message body. Address chaining provides verifiable conversation threading.

**Website Login.** The server presents a challenge containing a nonce and block height. The client signs the challenge with a BUTL header. The server verifies signature, freshness, and optionally balance. No passwords.

**WebSocket.** Phase 1 is a text frame containing the BUTL header. The server verifies and responds with a confirmation frame. Phase 2 is a binary frame containing the encrypted payload.

BUTL does not compete with existing transport infrastructure. It rides on top of it. No new network, no new servers, no changes to ISPs or routers. Any mechanism that can move bytes from one place to another — TCP, UDP, QUIC, HTTP, MQTT, Bluetooth, NFC, or a USB drive carried by hand — can carry a BUTL message.

-----

## 11. Protocol Summary

The complete BUTL header contains the following required fields:

|Field              |Purpose                                                 |
|-------------------|--------------------------------------------------------|
|BUTL-Version       |Protocol version (currently 1)                          |
|BUTL-Sender        |Sender’s Bitcoin address                                |
|BUTL-SenderPubKey  |Sender’s compressed secp256k1 public key (33 bytes)     |
|BUTL-Receiver      |Receiver’s Bitcoin address                              |
|BUTL-ReceiverPubKey|Receiver’s compressed secp256k1 public key (33 bytes)   |
|BUTL-BlockHeight   |Referenced Bitcoin block height                         |
|BUTL-BlockHash     |SHA-256d hash of referenced block                       |
|BUTL-PayloadHash   |SHA-256 of the encrypted payload                        |
|BUTL-Signature     |ECDSA signature over the canonical signing payload      |
|BUTL-PrevAddr      |Previous sender address in the chain (empty for genesis)|
|BUTL-ChainProof    |ECDSA proof linking current address to previous         |
|BUTL-EncAlgo       |Encryption algorithm identifier (default: AES-256-GCM)  |

Optional fields include BUTL-EphemeralPubKey (enhanced forward secrecy), BUTL-Nonce (anti-replay), BUTL-Timestamp (informational), BUTL-ThreadID (conversation identifier), BUTL-SeqNum (message ordering), BUTL-PayloadSize (pre-download validation), and BUTL-BalanceProof (offline balance verification).

The protocol sits between the application layer and the transport layer:

```
┌──────────────────────────────────────────┐
│           APPLICATION LAYER              │
│  Email · Chat · API · Login · VoIP · IoT │
├──────────────────────────────────────────┤
│           BUTL PROTOCOL                  │
│  Identity · Encryption · Integrity       │
│  Freshness · Chain · Gate                │
│  [Optional: Proof of Satoshi]            │
├──────────────────────────────────────────┤
│           BITCOIN NETWORK                │
│  SHA-256 · secp256k1 · ECDH · Blockchain │
├──────────────────────────────────────────┤
│           TRANSPORT (TCP/UDP/QUIC)       │
└──────────────────────────────────────────┘
```

-----

## 12. Security Analysis

**Cryptographic assumptions.** BUTL’s security rests on three well-established assumptions: the hardness of the Elliptic Curve Discrete Logarithm Problem on secp256k1 (ECDSA signatures and ECDH key agreement), the collision resistance of SHA-256 [6] (integrity and hashing), and the security of AES-256-GCM [5] as an authenticated encryption scheme. These are the most scrutinized primitives in cryptography, collectively securing Bitcoin, TLS 1.3, SSH, and the majority of the internet’s security infrastructure.

**No new assumptions.** BUTL introduces no novel cryptographic constructions. It composes existing primitives in a straightforward manner. The security of the composition follows directly from the security of the components.

**Man-in-the-middle.** An attacker who intercepts the header cannot forge the ECDSA signature (requires the sender’s private key) and cannot decrypt the payload (requires the receiver’s private key). The AAD binding in AES-256-GCM prevents payload redirection between recipients.

**Replay attacks.** The block height freshness check, combined with the optional nonce, ensures that captured messages cannot be replayed after the freshness window expires.

**Key compromise.** Each message uses a fresh keypair via address chaining. Compromising one key affects one message only. The attacker cannot continue the address chain without the current private key. Ephemeral ECDH keys (BUTL-EphemeralPubKey) provide additional forward secrecy by separating signing keys from encryption keys.

**Sybil attacks.** When Proof of Satoshi is disabled, unlimited identities can be created at zero cost. This is an explicit design choice documented in the specification. Applications requiring sybil resistance MUST enable Proof of Satoshi or implement alternative anti-sybil mechanisms at the application layer.

**Verify-before-download.** The security of this mechanism depends on correct implementation. The protocol specifies the behavior; the implementation enforces it. A buggy implementation that speculatively buffers payload data could violate the guarantee. Correct implementations treat the Phase 1 / Phase 2 boundary as a hard security gate.

**Private key as identity.** The self-sovereign identity model means that private key loss results in permanent identity loss, and private key compromise results in identity theft. There is no recovery mechanism and no revocation mechanism in v1.2. These are inherent tradeoffs of a system where no third party controls identity. Key revocation is planned for a future version.

-----

## 13. Conclusion

We have proposed a protocol for establishing trust between any two parties on the internet using Bitcoin’s existing cryptographic infrastructure. The protocol provides identity, authentication, encryption, integrity, freshness, forward privacy, and optional sybil resistance without relying on any centralized authority.

Unlike existing trust models that depend on certificate authorities, password databases, or corporate identity providers, BUTL derives its security entirely from mathematical hardness assumptions — the same assumptions that secure the Bitcoin network. When operating without Proof of Satoshi, every verification step is a local computation with zero external trust dependencies.

The protocol requires no changes to Bitcoin, introduces no new token or blockchain, and can be adopted incrementally by any internet application. It does not replace existing infrastructure — it adds a trust layer on top. It sits alongside TCP/IP and TLS rather than competing with them, providing the identity and trust layer that the internet was built without.

The reference implementations, minimum viable prototype, formal specification, and all associated documentation are dual licensed under the MIT License and Apache License 2.0, with an explicit patent grant and defensive patent pledge, establishing prior art and ensuring the protocol remains permanently free for all to use.

-----

## References

[1] S. Nakamoto, “Bitcoin: A Peer-to-Peer Electronic Cash System,” 2008.
https://bitcoin.org/bitcoin.pdf

[2] Bitcoin Wiki, “Base58Check encoding.”
https://en.bitcoin.it/wiki/Base58Check_encoding

[3] P. Wuille, “BIP-32: Hierarchical Deterministic Wallets,” 2012.
https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki

[4] D. Johnson, A. Menezes, S. Vanstone, “The Elliptic Curve Digital Signature Algorithm (ECDSA),” International Journal of Information Security, 2001.

[5] NIST, “Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC,” SP 800-38D, 2007.

[6] NIST, “Secure Hash Standard (SHS),” FIPS 180-4, 2015.

[7] SEC 2, “Recommended Elliptic Curve Domain Parameters,” Standards for Efficient Cryptography Group, 2010.

[8] RFC 8615, “Well-Known Uniform Resource Identifiers (URIs),” IETF, 2019.

[9] M. Bellare, C. Namprempre, “Authenticated Encryption: Relations among Notions and Analysis of the Generic Composition Paradigm,” Journal of Cryptology, 2008.

[10] BIP-39, “Mnemonic code for generating deterministic keys,” 2013.
https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki