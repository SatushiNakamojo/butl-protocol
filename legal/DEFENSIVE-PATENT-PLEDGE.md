# Defensive Patent Pledge

## BUTL Protocol — Irrevocable Commitment to Open Innovation

-----

## Purpose

This document is a binding, irrevocable pledge by all contributors to the BUTL (Bitcoin Universal Trust Layer) protocol. Its purpose is to ensure that the methods, algorithms, processes, and techniques described in the BUTL protocol can never be restricted, enclosed, or weaponized through patent litigation — by the creators, by future contributors, by corporate acquirers, or by anyone else.

The BUTL protocol is built on the principle that trust should come from math, not from institutions. The same principle applies to the protocol’s legal protection: its freedom comes from irrevocable commitments, not from institutional goodwill.

-----

## The Pledge

The creators and contributors of the BUTL protocol hereby pledge:

### 1. We Will Not Use Patents Offensively

No contributor to this repository will file patent applications on the methods, algorithms, processes, data structures, protocols, or techniques described in the BUTL protocol specification, its reference implementations, or its documentation.

If any contributor inadvertently obtains a patent covering BUTL methods — through employment, acquisition, merger, or any other means — that contributor pledges to license the patent royalty-free to all users of the BUTL protocol under the same terms as the [PATENTS](../PATENTS.md) grant.

This applies to all methods described in or implemented by the BUTL protocol, including but not limited to:

- The BUTL header format and field definitions
- The canonical signing payload construction and field ordering
- The 8-step verification sequence and its ordering
- The verify-before-download gate mechanism (two-phase delivery)
- The address chaining mechanism and chain proof construction
- The use of Bitcoin block height for message freshness
- The use of Bitcoin address balance for sybil resistance (Proof of Satoshi)
- The use of ECDH with Bitcoin secp256k1 keys for payload encryption
- The AAD binding of sender and receiver addresses in AES-256-GCM
- The `.well-known/butl.json` key discovery convention
- The BUTL handoff protocol for inter-application communication
- The toggle-and-slider design for optional Proof of Satoshi
- The BUTL-ID / BUTL-Contacts / spoke application architecture
- Any combination or extension of the above

### 2. We Will Defend the Commons

If any third party obtains a patent covering methods described in the BUTL protocol and attempts to enforce it against users or implementers of the protocol, the contributors to this repository pledge to:

- Provide evidence of prior art to invalidate the patent
- Cooperate with any party seeking to challenge the patent
- Make available the timestamped publication records (Git commits, GitHub releases, Wayback Machine archives, blockchain timestamps) as evidence
- Assist in preparing inter partes review (IPR) filings or equivalent proceedings in any jurisdiction

The prior art established by this repository — including all specifications, implementations, documentation, diagrams, and test vectors — predates any patent application filed after the initial commit date.

### 3. We Will Not Support Patent Trolls

No contributor will sell, assign, transfer, license, or otherwise convey any patent rights related to the BUTL protocol to any entity that:

- Has a history of offensive patent litigation
- Acquires patents primarily for the purpose of licensing or litigation
- Has been identified as a patent assertion entity (PAE) or non-practicing entity (NPE) engaged primarily in patent enforcement
- Would use the patents to restrict the use of the BUTL protocol by others

If a contributor’s employer, acquirer, successor, or assignee obtains patent rights related to BUTL methods, the contributor’s pledge survives and the patent rights remain royalty-free to all BUTL users.

### 4. This Pledge Is Irrevocable

This commitment:

- **Cannot be revoked** by any contributor, maintainer, or project owner
- **Cannot be modified** to reduce the protections it provides
- **Survives any transfer** of copyright, maintainership, or project ownership
- **Survives any corporate acquisition** — if a company acquires the project or a contributor’s employer, the pledge still applies
- **Survives any reorganization** — corporate mergers, spinoffs, bankruptcies, or dissolutions do not invalidate this pledge
- **Applies retroactively** to all contributions already made and prospectively to all future contributions
- **Binds successors and assigns** — anyone who acquires rights from a contributor inherits the obligations of this pledge

The only modification permitted is to strengthen the protections — never to weaken them.

-----

## How This Protects Users

If you implement the BUTL protocol in your software, this pledge (combined with the dual MIT/Apache 2.0 license and the PATENTS grant) provides three layers of protection:

### Layer 1: License-Level Protection

**Apache 2.0 users:** Section 3 of the Apache License 2.0 grants a perpetual, irrevocable patent license for all methods in the contribution. If anyone who contributed under Apache 2.0 sues you for patent infringement related to BUTL, they automatically lose their own patent license (the retaliation clause).

**MIT users:** The MIT License does not include patent language. MIT users receive protection from Layers 2 and 3.

### Layer 2: PATENTS.md Protection

The [PATENTS](../PATENTS.md) file provides an explicit, perpetual, worldwide, royalty-free, irrevocable patent grant covering all BUTL methods. This applies to **all users regardless of which license they choose**. It includes its own patent retaliation clause.

### Layer 3: This Pledge

This Defensive Patent Pledge provides:

- A commitment to never file patents on BUTL methods (offensive protection)
- A commitment to provide prior art evidence against third-party patents (defensive protection)
- A commitment to never sell patent rights to litigation entities (transfer protection)
- Irrevocability that survives corporate changes (durability protection)

### Combined Protection

|User’s License Choice|Layer 1                               |Layer 2                       |Layer 3                                                    |Total                   |
|---------------------|--------------------------------------|------------------------------|-----------------------------------------------------------|------------------------|
|Apache 2.0           |Patent grant + retaliation (Section 3)|PATENTS.md grant + retaliation|This pledge (offensive + defensive + transfer + durability)|Triple protection       |
|MIT                  |—                                     |PATENTS.md grant + retaliation|This pledge (offensive + defensive + transfer + durability)|Double protection       |
|Either               |Prior art from publication            |Prior art from publication    |Prior art from publication                                 |Prior art always applies|

Even if Layers 1 and 2 were somehow challenged or found insufficient (extremely unlikely), Layer 3 independently protects users. And even if all three layers failed (practically impossible), the prior art from the repository’s timestamped publication would invalidate any patent filed after the initial commit.

-----

## How This Binds Contributors

By submitting a pull request, issue, comment, code, documentation, or any other contribution to this repository, you agree to:

1. The terms of this Defensive Patent Pledge
1. The chosen license (MIT or Apache 2.0) for your contribution
1. The patent grant described in [PATENTS](../PATENTS.md)
1. The contributor requirements in [CONTRIBUTING](../CONTRIBUTING.md)

You represent that:

- You have the legal right to make these commitments
- Your contribution does not infringe any third-party patent that you are not willing to license under these terms
- You are not aware of any patent that would be infringed by your contribution, or if you are, you are willing to license it royalty-free under these terms
- Your employer (if applicable) has authorized you to make these commitments, or your contribution is made in a personal capacity

-----

## Prior Art Record

The following dates and artifacts establish prior art for the BUTL protocol. Any patent application filed after these dates that claims the methods described in this repository is anticipated by this prior art.

|Record                        |Date                  |Location                  |
|------------------------------|----------------------|--------------------------|
|Initial commit                |[DATE OF FIRST COMMIT]|GitHub commit hash: [HASH]|
|GitHub release v1.2.0         |2026 03 16            |https://github.com/satushinakamojo/butl-protocol|
|Internet Archive snapshot     |2026 03 16            |https://web.archive.org/web/20260316100403/https://github.com/SatushiNakamojo/butl-protocol|
|Bitcoin blockchain timestamp  |[DATE, if applicable] |Transaction ID: [TXID]    |
|arXiv / IACR ePrint submission|[DATE, if applicable] |[PREPRINT URL]            |

**Instructions for the maintainer:** Fill in these fields immediately after publishing the repository and completing each archival step. Each entry creates an independent, verifiable timestamp that can be used in patent proceedings.

The more records that exist, the stronger the prior art. Each record is independently verifiable by a third party and does not depend on the continued existence of any other record.

-----

## Enforcement

This pledge is self-enforcing through three mechanisms:

### 1. Community Enforcement

The open-source community has a strong track record of identifying and responding to patent threats against open projects. If a contributor violates this pledge, the community will know — the pledge is public, the contribution history is public, and the violation will be documented.

### 2. Legal Enforcement

This pledge, combined with the Apache 2.0 license (for users who choose it) and the PATENTS.md grant, creates a legally binding framework. A contributor who violates the pledge while having contributed under Apache 2.0 triggers the patent retaliation clause and loses their own patent license.

### 3. Prior Art Enforcement

Even if a contributor or third party somehow obtains a patent despite this pledge, the prior art from this repository’s publication provides grounds for invalidation through inter partes review (USPTO), opposition proceedings (EPO), or equivalent processes in other jurisdictions.

-----

## Relationship to Other Documents

|Document           |What It Provides                                                                |Who It Applies To              |
|-------------------|--------------------------------------------------------------------------------|-------------------------------|
|`LICENSE-MIT.md`   |Permissive copyright license (no patent language)                               |Users who choose MIT           |
|`LICENSE-APACHE.md`|Permissive copyright license + patent grant + retaliation (Section 3)           |Users who choose Apache 2.0    |
|`PATENTS.md`       |Explicit patent grant + retaliation (separate from license)                     |All users regardless of license|
|**This document**  |Pledge to never patent + defend the commons + never sell to trolls + irrevocable|All contributors and all users |

These four documents work together as a layered defense system. Each layer is independently valuable. Together, they provide the strongest possible protection for an open-source protocol.

-----

## Frequently Asked Questions

**Q: Can this pledge be revoked if the project changes hands?**

No. The pledge is irrevocable and survives any transfer of ownership, copyright, or maintainership. If the repository is transferred to a new owner, the new owner inherits the pledge. If a company acquires the project, the pledge still applies. This is by design.

**Q: What if a contributor’s employer files a patent on BUTL methods?**

The contributor’s pledge survives. The contributor is committed to licensing the patent royalty-free to all BUTL users. If the employer refuses, the prior art from this repository provides grounds for patent invalidation.

**Q: Does this pledge apply to methods I invent independently that happen to overlap with BUTL?**

Only if you contribute those methods to this repository. If you independently develop a method that overlaps with BUTL but never contribute it here, this pledge does not apply to your independent work. However, the prior art from this repository may still anticipate your patent application if the method was already described here.

**Q: What if I want to build a commercial product using BUTL?**

This pledge does not restrict commercial use. Both MIT and Apache 2.0 allow unrestricted commercial use. You can build, sell, and profit from BUTL-based products. This pledge only prevents you from using patents to restrict others from doing the same.

**Q: How is this different from the PATENTS.md file?**

The PATENTS.md file is a formal legal grant of patent rights. This pledge is a broader commitment that includes the promise to never file patents, to actively defend against third-party patents, to never sell rights to trolls, and to maintain irrevocability through corporate changes. They complement each other.

-----

## Signatures

By contributing to this repository, you agree to this pledge. Your contribution history in Git serves as your signature — timestamped, attributable, and permanent.

-----

*The protocol is free. This pledge ensures it stays free. Forever.*