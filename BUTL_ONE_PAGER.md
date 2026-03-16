# BUTL — Bitcoin Universal Trust Layer

### The internet has no trust layer. BUTL adds one.

-----

**The problem.** Every time you send an email, call an API, log into a website, or send a message, you’re trusting a middleman — a certificate authority, a password database, a corporate identity provider. These systems get hacked, go offline, censor users, and charge rent. The internet was built to move data, not to verify who sent it.

**The solution.** BUTL is an open protocol that uses Bitcoin’s cryptography to add identity, encryption, and trust to any internet communication. No new blockchain. No token. No company in the middle. Just the same math that secures over $1 trillion in Bitcoin, repurposed as a universal trust layer.

-----

### How It Works

Your identity is a Bitcoin address — a cryptographic key you generate yourself. No signup. No permission. No third party.

To send a BUTL message:

1. You **sign** it with your Bitcoin key — proves it’s you
1. You **encrypt** it to the receiver’s Bitcoin key — only they can read it
1. You include a recent **Bitcoin block height** — proves the message is fresh
1. You use a **new address** for each message, linked to your previous one — privacy with continuity

The receiver verifies everything **before downloading the payload**. If any check fails, the connection drops. Nothing touches their device.

-----

### What You Get

|Property            |How                        |What It Means                                               |
|--------------------|---------------------------|------------------------------------------------------------|
|**Identity**        |Bitcoin address (secp256k1)|You are your key. No signup. No CA.                         |
|**Authentication**  |ECDSA signature            |Only you can sign as you. Unforgeable.                      |
|**Encryption**      |ECDH + AES-256-GCM         |Only the intended receiver reads it.                        |
|**Integrity**       |SHA-256 payload hash       |Any tampering detected instantly.                           |
|**Freshness**       |Bitcoin block height       |Old messages rejected. Replay attacks fail.                 |
|**Privacy**         |Address chaining           |New address every message. Unlinkable by observers.         |
|**Device safety**   |Verify-before-download     |Nothing touches your device until verified.                 |
|**Sybil resistance**|Proof of Satoshi (optional)|Economic cost to fake identities. You control the threshold.|

-----

### Proof of Satoshi: Your Choice

The balance check is a receiver-side toggle with a configurable slider.

**Off:** BUTL is 100% pure mathematics. Zero external dependencies.

**On:** The receiver sets the minimum satoshi requirement. Senders who don’t meet it are rejected. Spam and bot farms become expensive.

The protocol provides the mechanism. The application chooses the policy.

-----

### Where It Works

Email. Messaging. API authentication. Website login. Video calls. IoT. Document signing. File transfer. Supply chain. Anywhere data moves between two parties.

BUTL doesn’t replace TCP/IP or TLS. It adds a trust layer on top. Existing infrastructure keeps working. Any transport that can move bytes can carry a BUTL message.

-----

### Prove It Yourself

```bash
python3 butl_mvp_v12.py
```

One command. Zero dependencies. Eight cryptographic proofs. Four Proof of Satoshi scenarios. Runs on any computer with Python 3. If it says `ALL 8 PROOFS PASSED`, the protocol works. On your machine. With your own eyes.

-----

### The Math

Three primitives. All battle-tested. No new cryptography.

|Primitive                    |Secures                             |
|-----------------------------|------------------------------------|
|**SHA-256**                  |Bitcoin, TLS, SSH, Git, PGP         |
|**ECDSA / ECDH on secp256k1**|Bitcoin ($1T+), Ethereum            |
|**AES-256-GCM**              |TLS 1.3, IPsec, SSH, every major VPN|

Without Proof of Satoshi, every verification step is a local computation. No network calls. No API trust. No external dependencies. Pure math.

-----

### Open Protocol

Dual licensed under **MIT** or **Apache 2.0**, at your option. Explicit patent grant. Defensive patent pledge. All methods published as prior art. Cannot be patented. Cannot be closed. Free forever.

**Website** butl-protocol.com (https://butl-protocol.com)

**GitHub:** github.com/satushinakamojo/butl-protocol (https://github.com/satushinakamojo/butl-protocol)

**White paper:** <BUTL_WHITE_PAPER.md>

**Specification:** <spec/BUTL_Protocol_Specification_v1.2.md>

-----

*BUTL — Because the internet deserves a trust layer that doesn’t trust anyone.*