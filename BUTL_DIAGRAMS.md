# BUTL Protocol Diagrams

## Visual Reference for the Bitcoin Universal Trust Layer

*14 diagrams covering every aspect of the protocol — from architecture to cryptography to ecosystem design.*

-----

## Diagram 1: Protocol Stack

Where BUTL sits in the network architecture.

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│               APPLICATION LAYER                     │
│                                                     │
│   Email   Chat   API   Login   VoIP   IoT   Files   │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│               BUTL PROTOCOL                         │
│                                                     │
│   ┌────────────┐ ┌────────────┐ ┌────────────┐     │
│   │  Identity  │ │ Encryption │ │ Integrity  │     │
│   │  (ECDSA)   │ │(ECDH + GCM)│ │ (SHA-256)  │     │
│   └────────────┘ └────────────┘ └────────────┘     │
│   ┌────────────┐ ┌────────────┐ ┌────────────┐     │
│   │ Freshness  │ │  Address   │ │    Gate    │     │
│   │(Block Hgt) │ │  Chaining  │ │(Verify 1st)│     │
│   └────────────┘ └────────────┘ └────────────┘     │
│   ┌──────────────────────────────────────────┐      │
│   │   [Optional] Proof of Satoshi (PoS)      │      │
│   └──────────────────────────────────────────┘      │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│               BITCOIN NETWORK                       │
│                                                     │
│   SHA-256   secp256k1   ECDH   Blockchain           │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│               TRANSPORT LAYER                       │
│                                                     │
│   TCP    UDP    QUIC    HTTP    MQTT    Bluetooth    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

-----

## Diagram 2: BUTL-ID — The Three Components

What a BUTL identity consists of and how the components relate.

```
┌────────────────────────────────────────────────────────────────┐
│                        BUTL-ID                                 │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│   ┌──────────────────┐                                         │
│   │   PRIVATE KEY    │   256-bit integer on secp256k1          │
│   │   (the secret)   │   Only you possess this.                │
│   │                  │   Never leaves your device.             │
│   │   4f3c2a1d8e...  │   Lost = identity gone forever.         │
│   └────────┬─────────┘                                         │
│            │                                                   │
│            │  Scalar multiplication: PubKey = PrivKey × G      │
│            │  (one-way: cannot reverse)                        │
│            ▼                                                   │
│   ┌──────────────────┐                                         │
│   │   PUBLIC KEY     │   Point on secp256k1 curve              │
│   │   (the tool)     │   33 bytes compressed (02/03 + x)       │
│   │                  │   Used for: signature verification,     │
│   │   02a1633caf...  │   ECDH encryption, chain proof checks   │
│   └────────┬─────────┘                                         │
│            │                                                   │
│            │  SHA-256 → RIPEMD-160 → Base58Check               │
│            │  (one-way: cannot reverse)                        │
│            ▼                                                   │
│   ┌──────────────────┐                                         │
│   │   ADDRESS        │   Human-readable identifier             │
│   │   (the name)     │   25-34 characters, checksummed         │
│   │                  │   Used for: identification, balance     │
│   │   1A1zP1eP5Q...  │   queries, AAD binding, contact books   │
│   └──────────────────┘                                         │
│                                                                │
│   Share the address and public key freely.                     │
│   NEVER share the private key.                                 │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

-----

## Diagram 3: Message Flow (Two-Phase Delivery)

The verify-before-download sequence between sender and receiver.

```
    SENDER                                            RECEIVER
      │                                                  │
      │  ┌───────────────────────────────────┐           │
      │  │ 1. Generate fresh keypair          │           │
      │  │ 2. ECDH with receiver's pubkey     │           │
      │  │ 3. AES-256-GCM encrypt body        │           │
      │  │ 4. Build canonical signing payload  │           │
      │  │ 5. ECDSA sign the payload          │           │
      │  │ 6. Create chain proof (if chained) │           │
      │  └───────────────────────────────────┘           │
      │                                                  │
      │  ════════ PHASE 1: HEADER ONLY ════════          │
      │                                                  │
      │──── BUTL Header (cleartext) ────────────────────>│
      │                                                  │
      │                      ┌───────────────────────────┤
      │                      │ BUTL GATE                 │
      │                      │                           │
      │                      │ Step 1: Structure    ✓/✗  │
      │                      │ Step 2: Receiver     ✓/✗  │
      │                      │ Step 3: Signature    ✓/✗  │
      │                      │ Step 4: Freshness    ✓/✗  │
      │                      │ Step 5: PoS     [optional] │
      │                      │ Step 6: Chain        ✓/✗  │
      │                      │                           │
      │                      │ ANY FAIL ──> DROP          │
      │                      │ ALL PASS ──> OPEN          │
      │                      └───────────────────────────┤
      │                                                  │
      │                            ┌───────────────┐     │
      │           IF GATE CLOSED:  │  Connection   │     │
      │                            │  terminated   │     │
      │                            │  Nothing      │     │
      │                            │  saved        │     │
      │                            └───────────────┘     │
      │                                                  │
      │  ════════ PHASE 2: PAYLOAD (conditional) ════════│
      │                                                  │
      │<────────────── BUTL-READY ───────────────────────│
      │                                                  │
      │──── Encrypted Payload ──────────────────────────>│
      │                                                  │
      │                      ┌───────────────────────────┤
      │                      │ POST-GATE                 │
      │                      │                           │
      │                      │ Step 7: Payload hash ✓/✗  │
      │                      │ Step 8: Decrypt GCM  ✓/✗  │
      │                      │                           │
      │                      │ FAIL ──> zero + discard   │
      │                      │ PASS ──> deliver to app   │
      │                      └───────────────────────────┤
      │                                                  │
      ▼                                                  ▼
```

-----

## Diagram 4: ECDH Encryption Flow

How sender and receiver derive the same shared secret.

```
      SENDER                                      RECEIVER
        │                                            │
   ┌────┴────┐                                  ┌────┴────┐
   │ privkey  │                                  │ privkey  │
   │ S_priv   │                                  │ R_priv   │
   └────┬────┘                                  └────┬────┘
        │                                            │
        │  × G                                       │  × G
        ▼                                            ▼
   ┌─────────┐                                  ┌─────────┐
   │ S_pub   │──── included in header ──────────│ R_pub   │
   └────┬────┘                                  └────┬────┘
        │                                            │
        │  S_priv × R_pub                            │  R_priv × S_pub
        │     ↓                                      │     ↓
        │  shared_point                              │  shared_point
        │  (same point!)                             │  (same point!)
        ▼                                            ▼
   ┌───────────────────┐                        ┌───────────────────┐
   │ SHA-256(point.x)  │                        │ SHA-256(point.x)  │
   │ = shared_secret   │ ════ IDENTICAL ═══════ │ = shared_secret   │
   └─────────┬─────────┘                        └─────────┬─────────┘
             │                                            │
             │  AES-256-GCM encrypt                       │  AES-256-GCM decrypt
             │  key = shared_secret                       │  key = shared_secret
             │  aad = sender || receiver                  │  aad = sender || receiver
             ▼                                            ▼
   ┌────────────────┐                           ┌────────────────┐
   │   Ciphertext   │──── transmitted ─────────>│   Plaintext    │
   └────────────────┘                           └────────────────┘


   WRONG RECEIVER (different private key):

   ┌─────────┐
   │ W_priv  │  × S_pub = DIFFERENT shared_point
   └────┬────┘
        ▼
   ┌────────────────────┐
   │ different secret   │──> AES-GCM decrypt ──> AUTH FAILED ✗
   └────────────────────┘
```

-----

## Diagram 5: Address Chaining

How identity continuity is maintained across fresh addresses.

```
   MESSAGE 0 (Genesis)         MESSAGE 1                 MESSAGE 2
   ┌───────────────────┐      ┌───────────────────┐     ┌───────────────────┐
   │  Key_0             │      │  Key_1             │     │  Key_2             │
   │  Addr_0            │      │  Addr_1            │     │  Addr_2            │
   │  PrevAddr: ""      │      │  PrevAddr: Addr_0  │     │  PrevAddr: Addr_1  │
   │  ChainProof: ""    │      │  ChainProof:       │     │  ChainProof:       │
   └────────┬──────────┘      │  Sign(Addr_1,      │     │  Sign(Addr_2,      │
            │                  │       Key_0)       │     │       Key_1)       │
            │                  └──────┬─────────────┘     └──────┬─────────────┘
            │                         │                          │
            ▼                         ▼                          ▼
   ┌─────────────┐           ┌─────────────┐           ┌─────────────┐
   │  1F4x...    │ ─────────>│  1K9y...    │ ─────────>│  1Bz7...    │
   └─────────────┘   proves  └─────────────┘   proves  └─────────────┘
                     same                      same
                     entity                    entity

   ThreadID = SHA-256(Addr_0)    ← constant across all messages


   WHAT AN OBSERVER SEES (without chain proofs):

   │  1F4x...  │     │  1K9y...  │     │  1Bz7...  │
   │  ???      │     │  ???      │     │  ???      │
   └───────────┘     └───────────┘     └───────────┘
        Three unrelated addresses. No way to link them.


   WHAT THE RECEIVER SEES (with chain proofs):

   │  1F4x...  │────>│  1K9y...  │────>│  1Bz7...  │
   │  SENDER   │     │  SENDER   │     │  SENDER   │
   └───────────┘     └───────────┘     └───────────┘
        Same entity. Cryptographically verified chain.
```

-----

## Diagram 6: Verification Gate (8 Steps)

The complete verification sequence with decision points.

```
                      ┌───────────────────┐
                      │   BUTL Header     │
                      │   arrives         │
                      └─────────┬─────────┘
                                │
                      ┌─────────▼─────────┐
                      │ Step 1:           │
                ┌─NO──│ Structure OK?     │
                │     └─────────┬─────────┘
                │               │ YES
                │     ┌─────────▼─────────┐
                │     │ Step 2:           │
                ├─NO──│ Receiver match?   │
                │     └─────────┬─────────┘
                │               │ YES
                │     ┌─────────▼─────────┐
                │     │ Step 3:           │
                ├─NO──│ Signature valid?  │
                │     └─────────┬─────────┘
                │               │ YES
                │     ┌─────────▼─────────┐
                │     │ Step 4:           │
                ├─NO──│ Block fresh?      │
                │     └─────────┬─────────┘
                │               │ YES
                │     ┌─────────▼─────────┐
                │     │ Step 5: (OPT)     │
                ├─NO──│ PoS balance OK?   │──── SKIP if disabled
                │     └─────────┬─────────┘
                │               │ YES / SKIP
                │     ┌─────────▼─────────┐
                │     │ Step 6:           │
                ├─NO──│ Chain proof OK?   │──── SKIP if genesis
                │     └─────────┬─────────┘
                │               │ YES / SKIP
                │               │
                │     ╔═════════▼═════════╗
                │     ║    GATE: OPEN     ║
                │     ║  Download payload ║
                │     ╚═════════╤═════════╝
                │               │
                │     ┌─────────▼─────────┐
                │     │ Step 7:           │
                ├─NO──│ Payload hash OK?  │
                │     └─────────┬─────────┘
                │               │ YES
                │     ┌─────────▼─────────┐
                │     │ Step 8:           │
                ├─NO──│ Decrypt OK?       │
                │     └─────────┬─────────┘
                │               │ YES
                │               │
                │     ╔═════════▼═════════╗
                │     ║  FULLY VERIFIED   ║
                │     ║  Deliver to app   ║
                │     ╚═══════════════════╝
                │
       ╔════════▼═════════╗
       ║    REJECTED      ║
       ║  Drop connection ║
       ║  Zero any data   ║
       ║  Nothing saved   ║
       ╚══════════════════╝
```

-----

## Diagram 7: BUTL Header Structure

Visual layout of a complete BUTL header.

```
┌───────────────────┬─────────────────────────────────────────────┐
│                   │           BUTL HEADER                       │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Version      │ 1                                           │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Sender       │ bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjh...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-SenderPubKey │ 02a1633cafcc01ebfb6d78e39f687a1f0995c6...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Receiver     │ bc1q9h5yjqka3mz7f3z8v5lgcd0cugaay3m6e...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-ReceiverPK   │ 03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-BlockHeight  │ 890412                                      │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-BlockHash    │ 00000000000000000002a7c4c1e48d76f0593d...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-PayloadHash  │ e3b0c44298fc1c149afbf4c8996fb92427ae41...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Signature    │ MEQCIGrD2w7h...  (base64 ECDSA)            │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-PrevAddr     │ bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-ChainProof   │ MEUCIQCv7n...  (base64 ECDSA)              │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-EncAlgo      │ AES-256-GCM                                 │
├───────────────────┼─────────────────────────────────────────────┤
│                   │         OPTIONAL FIELDS                     │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-EphemeralPK  │ 02f8e9d7c6b5a4938271605f4e3d2c1b0a...      │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Nonce        │ a4f8c3e91b2d                                │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-Timestamp    │ 1772937600                                  │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-ThreadID     │ 7f83b1657ff1fc53b92dc18148a1d65dfc2d4b...   │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-SeqNum       │ 3                                           │
├───────────────────┼─────────────────────────────────────────────┤
│ BUTL-PayloadSize  │ 4096                                        │
└───────────────────┴─────────────────────────────────────────────┘
```

-----

## Diagram 8: Encrypted Payload Structure

Binary layout of the encrypted payload.

```
┌────────────┬────────────────────────────────────┬────────────────┐
│     IV     │           Ciphertext               │   GCM Tag      │
│  12 bytes  │         variable length            │   16 bytes     │
└────────────┴────────────────────────────────────┴────────────────┘
│                                                                  │
│◄──────── BUTL-PayloadHash = SHA-256(entire blob) ──────────────►│
│                                                                  │

IV:          Random 12 bytes (nonce for AES-256-GCM)
Ciphertext:  AES-256-GCM(key = ECDH shared secret, plaintext = body)
GCM Tag:     16-byte authentication tag (integrity + authenticity)
AAD:         sender_address || receiver_address (not in payload, bound via GCM)
```

-----

## Diagram 9: Proof of Satoshi (Optional)

Receiver-configurable toggle and slider.

```
   RECEIVER CONFIGURATION:

   ┌──────────────────────────────────────────────┐
   │  Proof of Satoshi                            │
   │                                              │
   │  [ ● ] Enabled      [   ] Disabled           │
   │                                              │
   │  Minimum Balance:                            │
   │  ├──────────────●────────────────────────┤   │
   │  1           10,000                1,000,000  │
   │  sat           sat                    sat     │
   │                                              │
   │  Current setting: 10,000 satoshis            │
   └──────────────────────────────────────────────┘


   GATE BEHAVIOR:

   ┌───────────────────────────┐   ┌───────────────────────────┐
   │      PoS DISABLED         │   │      PoS ENABLED          │
   │                           │   │      min: 10,000 sat      │
   │  Step 1: Structure   ✓   │   │                           │
   │  Step 2: Receiver    ✓   │   │  Step 1: Structure   ✓   │
   │  Step 3: Signature   ✓   │   │  Step 2: Receiver    ✓   │
   │  Step 4: Freshness   ✓   │   │  Step 3: Signature   ✓   │
   │  Step 5: [SKIPPED]       │   │  Step 4: Freshness   ✓   │
   │  Step 6: Chain       ✓   │   │  Step 5: PoS ≥10k   ✓/✗ │
   │                           │   │  Step 6: Chain       ✓   │
   │  Gate: OPEN               │   │                           │
   │  Pure math. 0 queries.    │   │  Gate: OPEN or CLOSED     │
   │  Confidence: 99.7%        │   │  Requires BTC query.      │
   └───────────────────────────┘   │  Confidence: 97%          │
                                   └───────────────────────────┘
```

-----

## Diagram 10: Seed Phrase Recovery

What comes back and what doesn’t.

```
   ┌──────────────────────────────────────────────────────────┐
   │  12-WORD SEED PHRASE                                     │
   │  abandon ability able about above absent absorb ...      │
   └─────────────────────────┬────────────────────────────────┘
                             │
                             │  BIP-39 derivation
                             ▼
   ┌──────────────────────────────────────────────────────────┐
   │  MASTER SEED (32 bytes)                                  │
   └─────────────────────────┬────────────────────────────────┘
                             │
           ┌─────────────────┼─────────────────┐
           ▼                 ▼                  ▼
   ┌──────────────┐ ┌──────────────┐  ┌──────────────┐
   │  Key_0       │ │  Key_1       │  │  Key_N       │
   │  Address_0   │ │  Address_1   │  │  Address_N   │
   │  PubKey_0    │ │  PubKey_1    │  │  PubKey_N    │
   └──────────────┘ └──────────────┘  └──────────────┘

   ThreadID = SHA-256(Address_0)   ← also recovered


   ╔══════════════════════════════════════════════════════════╗
   ║  RECOVERED from seed phrase:                            ║
   ║  ✓ Every private key     ✓ Every public key             ║
   ║  ✓ Every address         ✓ Thread ID                    ║
   ║  ✓ Ability to sign       ✓ Ability to decrypt           ║
   ╠══════════════════════════════════════════════════════════╣
   ║  NOT RECOVERED from seed phrase:                        ║
   ║  ✗ Contact book          ✗ Chain state (current index)  ║
   ║  ✗ Conversation history  ✗ Personal fields on contacts  ║
   ║  ✗ Settings              ✗ Chain proofs from past msgs  ║
   ╚══════════════════════════════════════════════════════════╝
```

-----

## Diagram 11: Application Bindings

How BUTL integrates with different transport protocols.

```
   ┌──── EMAIL (SMTP) ──────────────────────────────────────────┐
   │                                                            │
   │  From: alice@example.com                                   │
   │  To: bob@example.com                                       │
   │  X-BUTL-Version: 1                                         │
   │  X-BUTL-Sender: bc1q...                                    │
   │  X-BUTL-Signature: MEQCIGr...                              │
   │  X-BUTL-PayloadHash: e3b0c4...                             │
   │  ... (all BUTL fields as X-headers)                        │
   │                                                            │
   │  [encrypted payload as MIME attachment]                     │
   └────────────────────────────────────────────────────────────┘

   ┌──── HTTP API ──────────────────────────────────────────────┐
   │                                                            │
   │  POST /api/data HTTP/1.1                                   │
   │  BUTL-Version: 1                                           │
   │  BUTL-Sender: bc1q...                                      │
   │  BUTL-Signature: MEQCIGr...                                │
   │  BUTL-PayloadHash: e3b0c4...                               │
   │                                                            │
   │  [encrypted payload as request body]                       │
   └────────────────────────────────────────────────────────────┘

   ┌──── WEBSOCKET ─────────────────────────────────────────────┐
   │                                                            │
   │  Frame 1 (text):    BUTL header as key: value lines        │
   │                ──>  Server verifies gate                    │
   │  Frame 2 (text):    "BUTL-READY"                           │
   │                <──  Server confirms gate open               │
   │  Frame 3 (binary):  Encrypted payload                      │
   │                ──>  Server decrypts                         │
   └────────────────────────────────────────────────────────────┘

   ┌──── KEY DISCOVERY ─────────────────────────────────────────┐
   │                                                            │
   │  GET https://example.com/.well-known/butl.json             │
   │                                                            │
   │  {                                                         │
   │    "version": 1,                                           │
   │    "address": "bc1q...",                                   │
   │    "pubkey": "02a163...",                                  │
   │    "pos_required": false,                                  │
   │    "freshness_window": 144                                 │
   │  }                                                         │
   └────────────────────────────────────────────────────────────┘
```

-----

## Diagram 12: Ecosystem Architecture

BUTL-Contacts as the hub, applications as spokes.

```
                    ┌────────────────────────┐
                    │        BUTL-ID         │
                    │                        │
                    │  Private key storage   │
                    │  Sign / ECDH / Backup  │
                    │  Restore from seed     │
                    └───────────┬────────────┘
                                │
                                │ API calls
                                ▼
                    ┌────────────────────────┐
                    │     BUTL-Contacts      │
                    │                        │
                    │  Contact book          │
                    │  Chain state tracking  │
                    │  Handoff engine        │
                    │  Settings              │
                    └─────┬──────┬──────┬────┘
                          │      │      │
              ┌───────────┘      │      └───────────┐
              │ handoff          │ handoff           │ handoff
              ▼                  ▼                   ▼
   ┌──────────────────┐ ┌──────────────┐ ┌──────────────────┐
   │   BUTL-Chat      │ │ BUTL-Video   │ │ BUTL-FileXfer    │
   │                  │ │              │ │                  │
   │  Encrypted       │ │  ECDH media  │ │  Verify-before-  │
   │  messaging       │ │  channels    │ │  download files  │
   └──────────────────┘ └──────────────┘ └──────────────────┘

              ┌───────────────────────────────┐
              │         BUTL-Login            │
              │                               │
              │  Sign challenges from         │
              │  websites. No passwords.      │
              └───────────────────────────────┘

   Every spoke app receives a BUTL handoff from Contacts.
   Every spoke app calls BUTL-ID for signing and encryption.
   No spoke app ever holds the private key.
```

-----

## Diagram 13: Licensing and Patent Protection

The layered protection model.

```
   ┌─────────────────────────────────────────────────────────┐
   │                                                         │
   │  User chooses:  MIT  ── OR ──  Apache 2.0               │
   │                                                         │
   │  ┌─────────────────┐       ┌────────────────────────┐   │
   │  │   MIT License   │       │  Apache License 2.0    │   │
   │  │                 │       │                        │   │
   │  │  Permissive     │       │  Permissive            │   │
   │  │  No patent      │       │  + Patent grant (§3)   │   │
   │  │  language        │       │  + Patent retaliation  │   │
   │  └────────┬────────┘       └──────────┬─────────────┘   │
   │           │                           │                 │
   │           └─────────┬─────────────────┘                 │
   │                     │                                   │
   │                     ▼                                   │
   │  ┌───────────────────────────────────────────────┐      │
   │  │  PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md      │      │
   │  │                                               │      │
   │  │  Apply to ALL users regardless of license     │      │
   │  │  Explicit patent grant + retaliation clause   │      │
   │  │  Irrevocable + prior art declaration          │      │
   │  └───────────────────────────────────────────────┘      │
   │                                                         │
   │  Apache 2.0 users:  Triple protection                   │
   │     (license patent grant + PATENTS + pledge)           │
   │                                                         │
   │  MIT users:  Double protection                          │
   │     (PATENTS + pledge)                                  │
   │                                                         │
   │  All users:  Prior art protection                       │
   │     (timestamped, archived, blockchain-provable)        │
   │                                                         │
   └─────────────────────────────────────────────────────────┘
```

-----

## Diagram 14: Security Properties Summary

What BUTL provides and which primitive delivers it.

```
┌───────────────────────────────────────────────────────────────────┐
│                    BUTL SECURITY PROPERTIES                      │
├─────────────────────┬───────────────────┬────────────────────────┤
│ Property            │ Primitive         │ Strength               │
├─────────────────────┼───────────────────┼────────────────────────┤
│ Identity            │ secp256k1 + B58   │ ECDLP (~2^128)         │
│ Authentication      │ ECDSA             │ EU-CMA                 │
│ Confidentiality     │ ECDH + AES-GCM   │ CDH + IND-CCA2         │
│ Integrity           │ SHA-256 + GCM     │ Collision (~2^128)     │
│ Freshness           │ Block height      │ Bitcoin PoW            │
│ Forward privacy     │ Address chain     │ ECDSA per message      │
│ Device safety       │ Gate mechanism    │ Implementation         │
│ Sybil resistance    │ Balance check     │ Economic (optional)    │
├─────────────────────┴───────────────────┴────────────────────────┤
│                                                                  │
│  Without PoS:  100% pure math. Zero external trust.              │
│                Confidence: 99.7%                                 │
│                                                                  │
│  With PoS:     Math + blockchain query.                          │
│                Confidence: 97%                                   │
│                                                                  │
│  Three primitives. All battle-tested:                            │
│    SHA-256    — secures Bitcoin, TLS, SSH, Git                   │
│    secp256k1  — secures $1T+ in Bitcoin                          │
│    AES-256-GCM — secures TLS 1.3, IPsec, every major VPN        │
│                                                                  │
│  No new cryptography. No new assumptions.                        │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

-----

*14 diagrams. Every aspect of the protocol visualized. From the math to the ecosystem.*