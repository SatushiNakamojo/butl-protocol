# Developer Warning: Proof of Satoshi Disabled

## What You Lose When the Balance Check Is Off — and When That’s Fine

-----

## The Warning

When Proof of Satoshi is disabled, BUTL provides identity, authentication, encryption, integrity, freshness, forward privacy, and verify-before-download. It does **not** provide sybil resistance.

This means an attacker can create unlimited BUTL identities at zero cost. Each identity will have a valid Bitcoin address, a valid public key, valid ECDSA signatures, valid ECDH encryption, valid chain proofs, and will pass every step of the verification gate except the one that is turned off.

The cryptography is correct. The identities are real. The math is sound. There is simply no economic barrier to creating them in bulk.

If your application is vulnerable to attacks that depend on creating many fake identities, you must either enable Proof of Satoshi or implement an alternative anti-sybil mechanism at the application layer.

**This is not a bug. It is a design decision. Proof of Satoshi is optional because not every application needs sybil resistance. But every developer should understand what “optional” means before choosing to disable it.**

-----

## What Sybil Resistance Prevents

A sybil attack is when an adversary creates many fake identities to gain disproportionate influence over a system. The name comes from the 1973 book about a person with multiple personalities.

In a BUTL context, sybil attacks look like this:

**Spam flooding.** An attacker generates 100,000 BUTL keypairs and sends messages from all of them. Each message has a valid signature, valid encryption, valid freshness, and passes the entire gate. The receiver’s application is overwhelmed with verified-but-unwanted messages.

**Vote manipulation.** An application uses BUTL identities for voting. One person creates 10,000 identities and votes 10,000 times. Each vote is cryptographically valid.

**Reputation gaming.** An application uses BUTL identities for reviews or ratings. An attacker creates fake identities to inflate or deflate ratings. Each identity passes verification.

**Resource exhaustion.** An attacker generates identities in bulk and uses them to consume API quotas, storage space, or computational resources. Each request is authenticated and passes the gate.

**Social graph pollution.** An attacker creates fake identities that contact real users, cluttering their contact books with synthetic entities that appear legitimate.

**Eclipse attacks.** In a peer-to-peer network, an attacker creates enough fake identities to surround a target node, controlling all of its connections.

In all of these scenarios, the attacker’s identities are cryptographically indistinguishable from legitimate ones. The signatures are real. The encryption works. The chain proofs are valid. The only thing missing is economic stake — and without Proof of Satoshi, that missing property isn’t checked.

-----

## Three Attack Scenarios in Detail

### Scenario 1: Bot Farm Against a Public API

**Setup:** A public REST API uses BUTL for authentication. Proof of Satoshi is disabled. Any valid BUTL identity can make requests.

**Attack:**

```
1. Attacker generates 50,000 keypairs (takes seconds on modern hardware)
2. Each keypair produces a valid Bitcoin address
3. Attacker sends API requests from all 50,000 identities
4. Each request has a valid BUTL signature, fresh block height, and valid header
5. The gate passes for every request
6. The API is overwhelmed with legitimate-looking traffic
```

**Cost to attacker:** Electricity to run `os.urandom(32)` fifty thousand times. Effectively zero.

**Why Proof of Satoshi stops this:** At a 1,000 satoshi threshold, the attacker needs 50,000 × 1,000 = 50,000,000 satoshis (0.5 BTC ≈ $50,000 at $100,000/BTC) locked in addresses plus transaction fees to fund each one. The economics make the attack irrational unless the API is worth more than $50,000 to abuse.

**Alternative mitigation without PoS:** Rate limiting per BUTL address. Allow N requests per address per time window. This doesn’t prevent the attacker from creating identities, but limits the damage each one can do. Works well for APIs, less well for voting or social systems.

### Scenario 2: Fake Users on a Social Platform

**Setup:** A social platform uses BUTL identities for accounts. Proof of Satoshi is disabled. Users can post, comment, and follow.

**Attack:**

```
1. Attacker creates 10,000 BUTL identities
2. Each identity creates a profile (valid BUTL-signed profile data)
3. Fake accounts follow a target user, inflating their follower count
4. Fake accounts post spam, misinformation, or harassment
5. Every action is signed by a valid BUTL identity
6. Moderation cannot distinguish fake accounts from real ones cryptographically
```

**Cost to attacker:** Near zero. Keypair generation is free.

**Why Proof of Satoshi stops this:** At 10,000 satoshis per identity, 10,000 fake accounts cost 100,000,000 satoshis (1 BTC ≈ $100,000). Creating a large bot army becomes a significant financial commitment. The attacker’s capital is locked and visible on-chain.

**Alternative mitigation without PoS:** Application-level reputation systems (account age, behavior scoring, manual verification). These work but are complex, gameable, and require ongoing moderation effort. Proof of Satoshi is simpler and more resistant to gaming.

### Scenario 3: Distributed Denial of Service via BUTL Messages

**Setup:** A messaging application uses BUTL. Proof of Satoshi is disabled. Any BUTL identity can send messages to any other.

**Attack:**

```
1. Attacker generates 1,000,000 keypairs
2. Each sends one BUTL message to the target
3. Each message passes the full 8-step gate
4. The target's device processes 1,000,000 verified messages
5. Storage, bandwidth, and CPU are consumed
6. Legitimate messages are buried
```

**Cost to attacker:** Minimal. The encrypted payloads are small. The keypair generation is instant.

**Why Proof of Satoshi stops this:** Even at 1 satoshi per identity, 1,000,000 identities require 1,000,000 on-chain transactions to fund. At typical fee rates, this costs thousands of dollars in transaction fees alone, plus the locked capital. The attack becomes prohibitively expensive.

**Alternative mitigation without PoS:** Application-level throttling (limit messages from unknown contacts), allowlists (only accept messages from known contacts), or proof-of-work challenges (require the sender to solve a computational puzzle before the gate runs). These work but add application complexity.

-----

## When Disabling PoS Is the Right Choice

Not every application is vulnerable to sybil attacks. Disabling Proof of Satoshi is correct when:

**The parties already know each other.** Two businesses exchanging authenticated API calls. Two people who have exchanged public keys in person. A device communicating with its own server. In these cases, the identity is established out-of-band, and creating fake identities is pointless because the receiver only accepts known contacts.

**The application doesn’t involve aggregation.** Voting, ratings, reputation, and follower counts are vulnerable to sybil attacks because they aggregate across identities. End-to-end encrypted messaging between two known parties is not — one fake identity can’t vote twice because there’s nothing to vote on.

**Offline operation is required.** Air-gapped systems, mesh networks, and embedded devices can’t query the blockchain. Proof of Satoshi requires connectivity. If the deployment environment has no internet, PoS must be disabled.

**Adoption breadth matters more than sybil resistance.** If the goal is maximum adoption (e.g., a universal login standard), requiring Bitcoin ownership restricts the user base. Disabling PoS allows anyone with a keypair to participate.

**Alternative anti-sybil mechanisms exist.** If the application already has rate limiting, allowlists, proof-of-work, CAPTCHAs, phone verification, or other identity gating, Proof of Satoshi may be redundant.

-----

## When Enabling PoS Is Essential

Enable Proof of Satoshi when:

- The application is publicly accessible (anyone can send)
- Actions aggregate across identities (voting, ratings, reputation)
- Creating many identities could overwhelm resources
- The application processes financial transactions
- Bot farms are a known threat in the application’s domain
- There is no alternative anti-sybil mechanism

-----

## Developer Checklist

Before deploying with Proof of Satoshi disabled, answer these questions:

- [ ] Can an attacker benefit from creating thousands of fake BUTL identities against this application?
- [ ] Does this application aggregate actions across identities (voting, ratings, follower counts)?
- [ ] Is this application publicly accessible to unknown senders?
- [ ] Could a flood of verified-but-fake messages degrade the user experience?
- [ ] Is there an alternative anti-sybil mechanism in place?

If any of the first four answers is **yes** and the fifth answer is **no**, Proof of Satoshi should be enabled.

-----

## Recommended Thresholds When Enabling

| Threat Level | Threshold | Use Case |
|-------------|-----------|----------|
| Minimal deterrence | 100 sat | IoT device registration, low-stakes APIs |
| Moderate deterrence | 1,000 sat | Public APIs, content submission |
| Strong deterrence | 10,000 sat | Social platforms, marketplace listings |
| Enterprise | 100,000 sat | Financial APIs, enterprise systems |
| High-value | 1,000,000 sat | High-value transactions, governance voting |
| Institutional | 10 BTC | Inter-institutional authentication |
| Whale | 100 BTC | Governance, proof of major holdings |
| Treasury | 1,000+ BTC | Exchange solvency, custodian verification |

The threshold should be set high enough that the attacker’s cost exceeds the value of the attack, but low enough that legitimate users can participate without significant financial commitment.

-----

## Alternative Anti-Sybil Mechanisms

If Proof of Satoshi cannot be enabled (offline deployment, non-Bitcoin users, regulatory constraints), consider these application-layer alternatives:

|Mechanism                |How It Works                                      |Strength                     |Weakness                        |
|-------------------------|--------------------------------------------------|-----------------------------|--------------------------------|
|Rate limiting per address|N requests per address per time window            |Simple, effective for APIs   |Attacker creates new addresses  |
|Allowlists               |Only accept messages from known contacts          |Strong for closed systems    |Doesn’t scale to open systems   |
|Proof of work            |Sender solves a puzzle before sending             |Economic cost without Bitcoin|CPU-intensive, mobile-unfriendly|
|Account age              |Require identity to exist for N days before acting|Slows drive-by attacks       |Attacker pre-generates accounts |
|Behavior scoring         |Reputation based on past actions                  |Adapts to real behavior      |Complex, gameable, slow to build|
|Phone/email verification |Out-of-band identity confirmation                 |Familiar to users            |Centralized, privacy-invasive   |
|CAPTCHA                  |Human verification challenge                      |Blocks simple bots           |Annoying, AI-solvable           |

None of these are as clean or as resistant to gaming as Proof of Satoshi. But all are better than nothing when PoS cannot be used.

-----

## The Bottom Line

Proof of Satoshi is optional because it should be optional. Not every application needs sybil resistance. But every developer deploying BUTL should make a conscious, informed decision about whether their application does.

The gate without Proof of Satoshi answers: “Is this message authentic, private, and fresh?” The gate with Proof of Satoshi also answers: “Does the sender have skin in the game?”

If your application only needs the first question answered, disable PoS with confidence. If it needs both, enable it and set the threshold to match your threat model. If you’re unsure, enable it at a low threshold (1,000 sat) — the cost to legitimate users is minimal, and the protection against bulk identity creation is immediate.

-----

*Disabling Proof of Satoshi is a valid choice. Make sure it’s a deliberate one.*