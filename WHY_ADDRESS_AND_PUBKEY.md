# Why Your BUTL-ID Has Both an Address and a Public Key

## They Look Related. They Are Related. But They Do Different Jobs.

-----

## The Short Answer

The **address** is your name tag. The **public key** is your working tool.

The address tells people who you are. The public key lets them do things with that identity — verify signatures, encrypt messages, and validate chain proofs.

Both are needed because one is derived from the other through a one-way process. Public key → address is always possible. Address → public key is mathematically impossible. That asymmetry is why both must exist in every BUTL header.

-----

## What Each One Is

### The Public Key

A point on the secp256k1 elliptic curve — the same curve Bitcoin uses. When compressed, it’s 33 bytes (66 hex characters). It looks like this:

```
02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc
```

The `02` at the start means the y-coordinate is even (`03` means odd). The remaining 64 hex characters are the x-coordinate. Together they define a unique point on the curve that corresponds to the private key.

**The public key is where the cryptography happens.** It is the input to every cryptographic operation in BUTL:

- **Signature verification.** The receiver feeds the public key, the signature, and the message hash into the ECDSA verification algorithm. Without the public key, there is no way to check whether a signature is valid.
- **Encryption.** The sender multiplies their private key by the receiver’s public key to compute an ECDH shared secret. Without the receiver’s public key, encryption to that specific recipient is impossible.
- **Chain proof verification.** The receiver uses the previous message’s sender public key to verify that the current address was signed by the same entity. Without the previous public key, chain continuity cannot be confirmed.

### The Address

Derived from the public key through a one-way hash chain:

```
Public Key (33 bytes)
    ↓  SHA-256
Hash (32 bytes)
    ↓  RIPEMD-160
Hash (20 bytes)
    ↓  Add version byte (0x00) + checksum (4 bytes)
    ↓  Base58Check encode
Address (25-34 characters)
```

It looks like this:

```
1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
```

The address is shorter, human-readable, and includes a built-in checksum that catches typos. It uses the same format as Bitcoin wallets, block explorers, and the entire Bitcoin ecosystem.

**The address is the identifier.** It is what gets shared with people, what appears in contact books, what the Proof of Satoshi balance check queries against, and what binds the encryption to the intended participants through the AAD.

-----

## Why Both Are Needed

### Why not just the address?

Because cryptography cannot be performed with an address. The hashing process that creates the address (SHA-256 followed by RIPEMD-160) is irreversible. Given only the address, there is no mathematical way to recover the public key.

Without the public key, the receiver cannot:

- Verify an ECDSA signature — the algorithm requires the curve point as input
- Compute an ECDH shared secret — elliptic curve multiplication requires the curve point
- Verify a chain proof — the algorithm requires the previous sender’s curve point

**If BUTL only included the address, every cryptographic operation in the protocol would be impossible.** The receiver would know who the message claims to be from but would have no way to verify, decrypt, or validate anything.

### Why not just the public key?

Technically, every cryptographic operation uses the public key. The address is derived from it. So why carry the address at all?

Five reasons:

**1. The Bitcoin ecosystem is indexed by address.** The Proof of Satoshi balance check queries a Bitcoin address, not a raw public key. Block explorers, Bitcoin full nodes, Electrum servers, and the entire UTXO set are organized by address. To check whether the sender holds satoshis, the address is needed.

**2. The address is human-readable.** A person can glance at `1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa` and recognize it, compare it to another address visually, copy it, paste it, read it aloud over the phone, or print it on a business card. `02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc` is none of these things. For contact books, logs, user interfaces, debugging, and casual communication, the address is the practical identifier.

**3. The address has built-in error detection.** Base58Check encoding includes a 4-byte checksum derived from double SHA-256. If a single character is mistyped, the checksum fails and the error is caught before any damage is done. Raw hex public keys have no such protection. A single wrong character silently corrupts every cryptographic operation that uses it — signatures fail, ECDH produces wrong shared secrets, chain proofs break — with no indication of why.

**4. The address provides defense-in-depth.** The address hides the public key behind two hash functions (SHA-256 + RIPEMD-160). If a future vulnerability is discovered in secp256k1 that allows computing a private key from a public key (no such vulnerability is known, but defense-in-depth matters), addresses that have never had their public key revealed on-chain remain protected. This is the same property that protects unspent Bitcoin UTXOs where the public key has not yet been exposed.

**5. The address binds the encryption.** In AES-256-GCM, the Additional Authenticated Data (AAD) is the concatenation of the sender’s address and the receiver’s address. This binds the ciphertext to both parties. If an attacker tries to redirect an encrypted payload to a different receiver, the AAD check fails and decryption is rejected. The addresses serve as a human-meaningful binding — not just “public key X encrypted to public key Y” but “this specific identity encrypted to that specific identity.”

**If BUTL only included the public key, the protocol would lose its connection to the Bitcoin ecosystem, its human readability, its error detection, its defense-in-depth, and its meaningful encryption binding.**

-----

## How They Work Together in BUTL

Every BUTL header carries both, for both sender and receiver:

```
BUTL-Sender:          1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
BUTL-SenderPubKey:    02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc
BUTL-Receiver:        bc1q9h5yjqka3mz7f3z8v5lgcd0cugaay3m6ez3065
BUTL-ReceiverPubKey:  03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0
```

The receiver uses them in different steps of the verification gate:

|Step               |Uses Address                             |Uses Public Key                           |
|-------------------|-----------------------------------------|------------------------------------------|
|1. Structure check |Validate format                          |Validate format (66 hex chars)            |
|2. Receiver match  |Compare `BUTL-Receiver` to own address   |—                                         |
|3. Signature       |—                                        |ECDSA verify with sender’s public key     |
|4. Freshness       |—                                        |—                                         |
|5. Proof of Satoshi|Query balance of sender’s address        |—                                         |
|6. Chain proof     |—                                        |Verify with previous sender’s public key  |
|7. Payload hash    |—                                        |—                                         |
|8. Decryption      |AAD binding (sender + receiver addresses)|ECDH shared secret (receiver’s public key)|

### The Consistency Check

The receiver also performs a critical consistency check: derive the address from the provided public key and confirm it matches the claimed address. If someone puts a valid public key but a mismatched address in the header, this check catches the inconsistency before any other verification proceeds.

```
derived = Base58Check(0x00 || RIPEMD-160(SHA-256(sender_pubkey)))

if derived != BUTL-Sender:
    REJECT — address does not match public key
```

This prevents an attacker from using one entity’s public key with a different entity’s address.

-----

## An Analogy

Think of a building:

- The **address** is the street address on the front of the building. It tells you which building it is. It’s what gets written on envelopes, printed on maps, and entered into navigation systems. It’s short, readable, and universally understood. But knowing the street address doesn’t let you open the door.
- The **public key** is the lock on the front door. It’s the mechanical part that determines who can and can’t get in. You need the lock to interact with the building — to verify that someone has the right key, to leave something securely inside, to prove that the contents haven’t been tampered with. But the lock by itself doesn’t tell you where the building is.

The street address finds the building. The lock secures it. Both are needed. BUTL works the same way.

-----

## Summary

|Property                         |Address                                   |Public Key                                |
|---------------------------------|------------------------------------------|------------------------------------------|
|Length                           |25-34 characters                          |66 hex characters (33 bytes)              |
|Format                           |Base58Check or Bech32                     |Compressed secp256k1 point (02/03 + x)    |
|Human readable                   |Yes                                       |No                                        |
|Error detection                  |Built-in 4-byte checksum                  |None                                      |
|Derived from                     |Public key (one-way hash)                 |Private key (one-way curve multiplication)|
|Used for ECDSA verification      |No                                        |Yes                                       |
|Used for ECDH encryption         |AAD binding only                          |Shared secret computation                 |
|Used for chain proof verification|No                                        |Yes                                       |
|Used for Proof of Satoshi        |Yes (blockchain query by address)         |No                                        |
|Used in contact books            |Yes (primary identifier)                  |Yes (stored for cryptographic operations) |
|Bitcoin ecosystem compatible     |Directly (same format)                    |Indirectly (derives to address)           |
|Defense-in-depth                 |Hides public key behind two hash functions|Exposed when used                         |

-----

## The One-Sentence Version

The address is who you are. The public key is how you prove it. Both are public. Both are safe to share. Both are necessary.

-----

*Two representations of the same identity. Different jobs. Both essential.*