# BUTL Ecosystem Architecture

## Contacts as the Hub. Applications as Spokes. BUTL-ID as the Foundation.

-----

## The Big Picture

BUTL is not a single application. It is an ecosystem of applications that share a common trust layer. Every application in the ecosystem — messaging, video calls, file transfer, website login, API authentication, document signing — uses the same cryptographic identity, the same verification gate, and the same address chaining mechanism.

The ecosystem is organized around three layers:

1. **BUTL-ID** — The identity foundation. Manages the private key. Performs all signing and ECDH operations. The private key never leaves this layer.
1. **BUTL-Contacts** — The identity hub. Manages the contact book, address chain state, settings, and the handoff engine that connects applications to each other.
1. **Spoke Applications** — The domain-specific tools. Chat, video, file transfer, login, email, document signing. Each receives a handoff from Contacts and focuses purely on its domain.

```
┌───────────────────────────────────────────────────────────┐
│                                                           │
│                        BUTL-ID                            │
│                                                           │
│   The identity foundation. Manages the private key.       │
│   API: sign(hash) → signature                             │
│        ecdh(their_pubkey) → shared_secret                 │
│        public_info() → {address, pubkey}                  │
│        backup() → seed_phrase                             │
│        restore(seed_phrase) → ok                          │
│                                                           │
│   Private key NEVER leaves this boundary.                 │
│                                                           │
└──────────────────────────┬────────────────────────────────┘
                           │
                           │ API calls (sign, ecdh, public_info)
                           │
┌──────────────────────────▼────────────────────────────────┐
│                                                           │
│                     BUTL-Contacts                         │
│                                                           │
│   The identity hub. Knows who you are and who you know.   │
│                                                           │
│   ┌─────────────┐ ┌──────────────┐ ┌──────────────────┐  │
│   │ Contact Book │ │ Chain State  │ │ Handoff Engine   │  │
│   │             │ │              │ │                  │  │
│   │ Names       │ │ Current idx  │ │ Constructs JSON  │  │
│   │ Addresses   │ │ Thread IDs   │ │ handoff with     │  │
│   │ Public keys │ │ Seq numbers  │ │ sender + receiver│  │
│   │ Personal    │ │ Previous     │ │ context for any  │  │
│   │ fields      │ │ sender keys  │ │ spoke app        │  │
│   └─────────────┘ └──────────────┘ └────────┬─────────┘  │
│                                              │            │
│   ┌──────────────────┐                       │            │
│   │ Settings         │                       │            │
│   │                  │                       │            │
│   │ PoS toggle       │                       │            │
│   │ PoS slider       │                       │            │
│   │ Freshness window │                       │            │
│   │ Default action   │                       │            │
│   └──────────────────┘                       │            │
│                                              │            │
└──────────────────────────────────────────────┼────────────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    │                          │                          │
                    │ handoff                  │ handoff                  │ handoff
                    ▼                          ▼                          ▼
         ┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
         │   BUTL-Chat      │       │   BUTL-Video     │       │  BUTL-FileXfer   │
         │                  │       │                  │       │                  │
         │  Encrypted       │       │  ECDH-encrypted  │       │  Verify-before-  │
         │  real-time       │       │  media streams   │       │  download file   │
         │  messaging       │       │  with identity   │       │  transfer with   │
         │                  │       │  verification    │       │  integrity       │
         └──────────────────┘       └──────────────────┘       └──────────────────┘

         ┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
         │   BUTL-Login     │       │   BUTL-Email     │       │   BUTL-Sign      │
         │                  │       │                  │       │                  │
         │  Passwordless    │       │  BUTL headers    │       │  Document        │
         │  website auth    │       │  on SMTP with    │       │  signing with    │
         │  via signed      │       │  encrypted       │       │  blockchain      │
         │  challenges      │       │  attachments     │       │  timestamps      │
         └──────────────────┘       └──────────────────┘       └──────────────────┘
```

-----

## Why This Architecture

### Why Not One Monolithic App?

A single app that does messaging, video, file transfer, login, and email would be:

- **Massive.** Hundreds of thousands of lines of code. Huge attack surface.
- **Slow to develop.** Every feature blocks every other feature.
- **Fragile.** A bug in the video codec could crash the messaging system.
- **Unextensible.** Adding a new use case means modifying the monolith.
- **All or nothing.** Users who only want login are forced to install a messaging stack.

The hub-and-spoke model avoids all of these problems. Each spoke is small, focused, and independently developed. A bug in BUTL-Chat cannot affect BUTL-Login. A new spoke can be added without modifying any existing component.

### Why Contacts as the Hub?

Every communication starts with the same two questions:

1. **Who am I?** (my keypair, my address, my chain state)
1. **Who am I talking to?** (their address, their public key, their chain state)

These questions are answered by the contact book. Whether the user wants to chat, video call, transfer a file, or log into a website, they first select a contact (or the contact’s identity is discovered via `.well-known/butl.json`). Contacts is the natural starting point for every interaction.

The handoff engine then constructs a BUTL context (a JSON blob containing everything the spoke app needs) and passes it to the appropriate application. The spoke app never needs to implement identity management, contact lookup, or chain state tracking — it receives all of that from Contacts.

### Why BUTL-ID as a Separate Layer?

The private key is the most sensitive data in the entire ecosystem. By isolating it in its own layer:

- **Minimal attack surface.** BUTL-ID manages exactly one thing: the encrypted keypair. No contact parsing, no network protocol handling, no UI rendering.
- **Process isolation.** The private key exists in BUTL-ID’s process memory only. Neither Contacts nor any spoke app ever holds the raw key.
- **Hardware portability.** BUTL-ID can be implemented on a hardware security module (HSM), secure enclave (ARM TrustZone, Apple Secure Enclave), or hardware wallet (COLDCARD, Ledger, Trezor). The private key never leaves the hardware. The calling app sends a hash to sign; the hardware returns the signature.
- **Independent update cadence.** BUTL-ID changes rarely (it’s just key operations). Contacts and spoke apps change frequently. Separating them means the most critical component is the most stable.

-----

## The Handoff Protocol

The handoff is the bridge between Contacts and every spoke application. It is a standardized JSON blob that Contacts constructs and passes to a spoke app, containing everything the spoke needs to communicate with a specific contact.

### Handoff Structure

```json
{
  "handoff_version": 1,
  "action": "chat",
  "timestamp": 1772937600,
  "sender": {
    "address": "1MyAddress...",
    "pubkey": "02my_pubkey...",
    "keychain_ref": "~/.butl/identity.enc",
    "chain_index": 6
  },
  "receiver": {
    "name": "Bob Smith",
    "address": "1BobsAddress...",
    "pubkey": "02bobs_pubkey...",
    "last_sender_pubkey": "02last_msg_pubkey...",
    "thread_id": "7f83b165...",
    "seq_num": 5
  },
  "config": {
    "pos_enabled": false,
    "pos_min_satoshis": 0,
    "freshness_window": 144
  }
}
```

### Field Definitions

|Field            |Purpose                                                                              |
|-----------------|-------------------------------------------------------------------------------------|
|`handoff_version`|Handoff protocol version (currently 1)                                               |
|`action`         |Which spoke app to invoke: `chat`, `video`, `file`, `login`, `email`, `sign`         |
|`timestamp`      |When the handoff was created (for freshness — reject stale handoffs)                 |
|`sender`         |The user’s identity: address, pubkey, keychain reference, current chain index        |
|`receiver`       |The contact’s identity: name, address, pubkey, last known sender pubkey, thread state|
|`config`         |BUTL gate configuration for this interaction                                         |

### What the Handoff Does NOT Contain

The handoff does NOT contain the private key. It contains a reference to the encrypted identity file (`keychain_ref`). The spoke app calls BUTL-ID to perform signing and ECDH operations. BUTL-ID prompts for the password (if needed) and returns only the result — never the key.

### Delivery Methods

|Method        |When to Use     |How It Works                                                                                 |
|--------------|----------------|---------------------------------------------------------------------------------------------|
|**File**      |CLI applications|Contacts writes handoff to a temp file with 0600 permissions. Spoke app reads and deletes it.|
|**URI scheme**|GUI applications|`butl://chat?handoff=base64_encoded_json`. Spoke app registers as URI handler.               |
|**Pipe**      |Programmatic use|`butl-contacts handoff "Bob" --action chat --stdout | butl-chat --stdin`                     |
|**IPC**       |System service  |Unix domain socket, named pipe, or D-Bus/XPC message.                                        |

### Handoff Security

- Private key is never included — only a keychain reference.
- Temp files use restricted permissions (0600) and are deleted after read.
- Handoffs include a timestamp and should be rejected if older than 60 seconds.
- The handoff is not transmitted over a network — it moves between applications on the same device.

-----

## How the Pieces Interact

### Scenario: Sending a Chat Message

```
User opens BUTL-Contacts
    │
    ├── Selects "Bob Smith" from contact book
    │
    ├── Taps "Chat"
    │
    ├── Contacts constructs a handoff:
    │   {action: "chat", receiver: {Bob's identity + chain state}, ...}
    │
    ├── Contacts launches BUTL-Chat with the handoff
    │
    └── BUTL-Chat opens

BUTL-Chat receives the handoff
    │
    ├── Extracts Bob's address, pubkey, thread state
    │
    ├── User types "Hello Bob!"
    │
    ├── Chat calls BUTL-ID:
    │   ├── ecdh(Bob's pubkey) → shared secret
    │   ├── encrypt(body, shared_secret, AAD)
    │   └── sign(canonical_hash) → signature
    │
    ├── Chat constructs the BUTL header + encrypted payload
    │
    ├── Chat transmits Phase 1 (header) then Phase 2 (payload)
    │
    └── Chat updates chain state → passes back to Contacts
```

### Scenario: Receiving a Chat Message

```
BUTL-Chat receives an incoming connection
    │
    ├── Reads BUTL header (Phase 1)
    │
    ├── Calls BUTL-ID to verify:
    │   ├── Signature verification (needs sender's pubkey from header)
    │   └── ECDH shared secret (needs receiver's private key via BUTL-ID)
    │
    ├── Queries Contacts for the sender's previous pubkey (chain proof verification)
    │
    ├── Runs the full 8-step gate
    │
    ├── If gate passes → requests payload (Phase 2)
    │
    ├── Decrypts and displays: "Hello Bob!"
    │
    └── Updates chain state in Contacts (new sender pubkey, incremented seq_num)
```

### Scenario: Passwordless Website Login

```
User visits https://example.com → server presents BUTL challenge
    │
    ├── Browser extension (BUTL-Login) detects the challenge
    │
    ├── BUTL-Login queries Contacts: "Do I have a key for example.com?"
    │   ├── If yes → retrieves the contact entry
    │   └── If no → fetches https://example.com/.well-known/butl.json
    │
    ├── BUTL-Login constructs a BUTL response:
    │   ├── Calls BUTL-ID: sign(challenge_hash) → signature
    │   └── Builds BUTL header with signature + freshness
    │
    ├── Sends BUTL header to server
    │
    ├── Server verifies signature → user is authenticated
    │
    └── No password. No OAuth. No third party.
```

-----

## Data Ownership

### What Lives Where

|Data                                                            |Where It Lives                    |Why                                                            |
|----------------------------------------------------------------|----------------------------------|---------------------------------------------------------------|
|Private key (encrypted)                                         |BUTL-ID                           |Most sensitive. Smallest possible attack surface. Never leaves.|
|Public key + address                                            |BUTL-ID, BUTL-Contacts, spoke apps|Public information. Safe to copy.                              |
|Contact book                                                    |BUTL-Contacts                     |Social graph. Encrypted at rest.                               |
|Chain state (current index, previous sender pubkeys, thread IDs)|BUTL-Contacts                     |State needed to verify the next message in a chain.            |
|Settings (PoS toggle, freshness window)                         |BUTL-Contacts                     |User preferences. Not secrets.                                 |
|Message content                                                 |Spoke apps (Chat, Email, etc.)    |Application-level data. Not protocol-level.                    |
|Handoff (transient)                                             |Temp file / pipe / IPC            |Exists only for the moment of transfer. Deleted after read.    |

### Data Flow Rules

1. **Private key flows nowhere.** BUTL-ID holds it. Everything else calls BUTL-ID’s API.
1. **Public identity flows freely.** Address and pubkey are shared with anyone who needs them.
1. **Contact data flows from Contacts to spoke apps via handoff.** Spoke apps receive what they need and nothing more.
1. **Chain state flows back from spoke apps to Contacts after each message.** The spoke app reports the new sender pubkey and sequence number so Contacts can update its records.
1. **Message content stays in the spoke app.** Contacts never sees the plaintext of a chat message, the contents of a transferred file, or the body of an email.

-----

## Adding a New Spoke Application

The architecture is designed to be extensible. Adding a new spoke requires:

### 1. Define the Action

Choose an action name for the handoff: `chat`, `video`, `file`, `login`, `email`, `sign`, or a new custom action.

### 2. Parse the Handoff

The spoke app receives the handoff JSON and extracts the sender identity, receiver identity, chain state, and gate configuration.

### 3. Implement the Domain Logic

The spoke app implements its specific functionality (messaging UI, video codec, file transfer protocol, etc.) using the BUTL context from the handoff.

### 4. Call BUTL-ID for Cryptography

All signing and ECDH operations go through BUTL-ID. The spoke app sends a hash, receives a signature. Sends a public key, receives a shared secret. Never touches the private key.

### 5. Update Chain State

After each message exchange, the spoke app sends the updated chain state (new sender pubkey, new sequence number) back to Contacts.

### 6. Register as a Handler

For GUI deployments, register as a handler for the `butl://` URI scheme with the relevant action. For CLI deployments, accept a `--handoff` flag pointing to the handoff file.

That’s it. The spoke app doesn’t implement identity management, contact lookup, key storage, backup, restore, or chain state tracking. All of that is handled by BUTL-ID and Contacts.

-----

## The URI Scheme

For GUI applications, Contacts launches spoke apps via a URI:

```
butl://chat?handoff=eyJoYW5kb2ZmX3Zlc...
butl://video?handoff=eyJoYW5kb2ZmX3Zlc...
butl://file?handoff=eyJoYW5kb2ZmX3Zlc...
butl://login?handoff=eyJoYW5kb2ZmX3Zlc...
```

The scheme is `butl://`. The host is the action name. The `handoff` query parameter contains the Base64-encoded handoff JSON. The spoke app registered as the handler for that action receives the URI, decodes the handoff, and begins operation.

-----

## Comparison to Existing Ecosystems

|Property             |BUTL Ecosystem                        |Apple Ecosystem                        |Google Ecosystem                             |
|---------------------|--------------------------------------|---------------------------------------|---------------------------------------------|
|Identity root        |Owner’s secp256k1 key                 |Apple ID (corporate account)           |Google Account (corporate account)           |
|Key storage          |BUTL-ID (local, encrypted)            |Keychain (local + iCloud)              |Google Password Manager (cloud)              |
|Contact management   |BUTL-Contacts (local, encrypted)      |Apple Contacts (iCloud sync)           |Google Contacts (cloud)                      |
|App handoff          |BUTL handoff protocol (JSON, local)   |Apple Handoff (Bluetooth + iCloud)     |Android Intents (local)                      |
|Who controls identity|Device owner                          |Apple                                  |Google                                       |
|Encryption           |End-to-end (ECDH, owner controls keys)|Mixed (some E2E, Apple holds some keys)|Mixed (Google holds most keys)               |
|Works offline        |Yes (pure math)                       |Partially (some features need iCloud)  |Partially (most features need cloud)         |
|Open standard        |Yes (MIT OR Apache 2.0)               |No (proprietary)                       |Partially (Android is open, services are not)|
|Third-party spokes   |Anyone can build one                  |Apple approval required                |Google Play approval recommended             |

The BUTL ecosystem provides the same user experience (identity hub + application spokes + seamless handoff) but with self-sovereign identity, end-to-end encryption the owner controls, and no corporate gatekeeper.

-----

## Build Order

The ecosystem is built in phases, each independently useful:

### Phase 1: BUTL-Contacts (The Hub)

The identity manager, contact book, and handoff engine. CLI-first. Everything else depends on this. When Phase 1 is complete, a user can create an identity, manage contacts, and generate handoffs.

### Phase 2: BUTL-Chat (First Spoke)

The first application that receives a handoff and does something with it. Real-time encrypted messaging between two parties. Proves the handoff protocol works, the two-phase gate works in real time, and address chaining advances correctly across a conversation.

### Phase 3: BUTL-ID (Extraction)

Extract identity management from Contacts into a standalone service. BUTL-ID manages the private key. Contacts and Chat call BUTL-ID for signing and ECDH. The private key never exists in any other process’s memory.

### Phase 4: Additional Spokes

BUTL-Login (browser extension), BUTL-FileTransfer, BUTL-Email, BUTL-VideoChat, BUTL-Sign. Each is independent. Each uses the same handoff protocol. Each calls BUTL-ID for cryptography.

### Phase 5: GUI Layer

Desktop and mobile interfaces for Contacts and all spoke apps. The CLI applications from earlier phases serve as the backend. The GUI is a thin layer on top.

-----

## The Principle

The ecosystem follows one principle at every level:

**Separate what changes from what doesn’t.**

- BUTL-ID (the private key) changes never. It is generated once and used forever.
- BUTL-Contacts (the social graph) changes slowly. Contacts are added and removed over weeks and months.
- Spoke apps (the domain logic) change frequently. New features, new protocols, new UI.
- Messages (the content) change constantly. Every conversation is different.

By separating these layers, the most critical component (the key) is the most stable, and the most actively developed components (the apps) can evolve without risking the identity foundation.

-----

*One identity. One contact book. Many applications. The hub holds the trust. The spokes deliver the value.*