# Versioning and Backward Compatibility

## How BUTL Protocol Versions Work

This document defines how the BUTL protocol is versioned, how receivers handle version mismatches, how interactive protocols negotiate versions, what changes require a version increment, and how the ecosystem migrates between versions.

-----

## Two Kinds of Version Numbers

BUTL has two version numbers that serve different purposes. Understanding the distinction is important.

### Protocol Version

An integer carried in the `BUTL-Version` header field. This is the version that matters on the wire — it tells the receiver which rules to use for verification. The current protocol version is **1**.

The protocol version increments only when the wire format changes in a way that breaks backward compatibility. All messages with `BUTL-Version: 1` are verified using the same rules, regardless of which spec revision produced them.

### Spec Revision

A semantic version number (e.g., v1.0, v1.1, v1.2) that tracks documentation and feature changes. The spec revision appears in document titles and the CHANGELOG but is NOT transmitted in the header.

Multiple spec revisions can share the same protocol version. This is the current situation:

|Spec Revision|Protocol Version|What Changed                                                        |
|-------------|----------------|--------------------------------------------------------------------|
|v1.0         |1               |Initial specification                                               |
|v1.1         |1               |Added encryption fields, verify-before-download, ephemeral keys     |
|v1.2         |1               |Made Proof of Satoshi optional, dual license, expanded documentation|

All three spec revisions produce messages with `BUTL-Version: 1`. A receiver that supports protocol version 1 can verify messages from any of them.

-----

## What Triggers a Protocol Version Increment

A new protocol version (`BUTL-Version: 2`) is REQUIRED when any of the following change:

|Change                                                                        |Why It Breaks Compatibility                                                       |
|------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
|Canonical signing payload format changes (fields added, removed, or reordered)|Sender and receiver would compute different hashes, causing all signatures to fail|
|A required header field is added                                              |Older receivers don’t know to check the new field                                 |
|A required header field is removed                                            |Older receivers still expect the field and reject messages without it             |
|The meaning of an existing required field changes                             |Older receivers interpret the value incorrectly                                   |
|The verification step order changes                                           |Older receivers run the wrong sequence                                            |
|The encryption scheme changes in a non-backward-compatible way                |Older receivers can’t decrypt                                                     |

A new protocol version is NOT required when:

|Change                                         |Why It Doesn’t Break Compatibility                               |
|-----------------------------------------------|-----------------------------------------------------------------|
|An optional header field is added              |Receivers ignore unrecognized fields (forward compatibility rule)|
|Documentation is updated                       |No wire-format change                                            |
|Recommended thresholds or best practices change|Advisory, not protocol-level                                     |
|New application bindings are documented        |Transport-layer, not protocol-level                              |
|Proof of Satoshi configuration changes         |Receiver-side setting, not header-level                          |

-----

## Receiver Behavior on Version Mismatch

### Message version equals receiver’s supported version

Process normally. This is the expected case.

### Message version is higher than receiver supports

**REJECT the message.** The receiver MUST NOT attempt to verify a message with a version it does not understand. The canonical signing payload format or verification steps may have changed, and attempting verification would produce incorrect results — either false rejections (rejecting valid messages) or, worse, false acceptances (accepting invalid messages).

The receiver SHOULD return or log a descriptive error:

```
BUTL-Error: Unsupported version 2. This implementation supports version 1.
```

### Message version is lower than receiver supports

**Process using the rules for that version.** A receiver that supports version 2 and receives a version 1 message SHOULD verify it using version 1 rules. This allows gradual migration — not all senders upgrade simultaneously.

Implementations SHOULD maintain support for at least one previous protocol version to allow a reasonable migration window.

### Message version is zero or negative

**REJECT.** Version 0 and negative versions are not defined. This is likely a malformed or malicious header.

-----

## Version Negotiation (Interactive Protocols)

For non-interactive protocols (email, one-shot API calls), there is no negotiation. The sender includes `BUTL-Version: 1` and the receiver either supports it or rejects it. This is the simple case and covers most BUTL usage.

For interactive protocols (WebSocket, TCP, persistent HTTP connections), negotiation allows both parties to agree on the highest mutually supported version before exchanging messages.

### Negotiation Flow

```
Client → Server:   BUTL-Version-Supported: 1,2
Server → Client:   BUTL-Version-Selected: 2

(All subsequent messages in this session use version 2)
```

### No Overlap

```
Client → Server:   BUTL-Version-Supported: 3
Server → Client:   BUTL-Error: No compatible version. Server supports: 1,2
                   (Connection closed)
```

### Rules

- The `BUTL-Version-Supported` field contains a comma-separated list of supported protocol versions, in ascending order.
- The server selects the highest version that both parties support.
- If there is no overlap, the server returns an error and closes the connection.
- After negotiation, all messages in the session MUST use the selected version.
- Negotiation occurs once per session, not per message.
- Negotiation is OPTIONAL. If neither party sends `BUTL-Version-Supported`, both assume version 1.

-----

## Forward Compatibility Rules

These rules ensure that new features can be introduced without breaking existing implementations.

### Rule 1: Ignore Unknown Optional Headers

If a receiver encounters a `BUTL-*` header field it does not recognize, it MUST ignore it and continue verification. This allows new optional fields to be introduced in future spec revisions without requiring all implementations to update simultaneously.

```
Example: A v1.2 implementation sends BUTL-EphemeralPubKey.
A v1.0 implementation that doesn't know about ephemeral keys
simply ignores the field and uses BUTL-SenderPubKey for ECDH.
The message still verifies correctly.
```

### Rule 2: Reject Unknown Protocol Versions

If `BUTL-Version` is higher than what the receiver supports, REJECT immediately. Do not attempt partial verification. The rules may have changed in ways that make partial verification unreliable.

### Rule 3: Preserve Unknown Headers on Relay

If an intermediary (e.g., an email server, a message relay, a load balancer) forwards a BUTL message, it MUST preserve all `BUTL-*` headers, including those it does not recognize. Stripping unknown headers would break verification for receivers that do understand them.

### Rule 4: Tolerate Missing Optional Fields

A receiver MUST NOT reject a message because an optional field is absent. Optional means optional. The verification sequence must function correctly whether optional fields are present or not.

-----

## Migration Strategy

When a new protocol version is released, the ecosystem migrates in three phases:

### Phase 1: Dual Support (Months 1-12)

Implementations add support for the new version while continuing to support the old version. During this phase:

- Senders can be configured to produce either version based on what the receiver supports
- Receivers accept and correctly verify both versions
- Version negotiation (for interactive protocols) automatically selects the highest mutual version
- No one is forced to upgrade immediately

### Phase 2: Deprecation (Months 12-24)

The old version is marked as deprecated. This means:

- The old version still works but is no longer recommended
- New features and improvements are only available in the new version
- Security fixes are backported to the old version during this phase
- Documentation encourages migration with specific upgrade instructions

### Phase 3: Sunset (After Month 24)

The old version is no longer supported. This means:

- Implementations MAY drop support for the old version
- Security fixes are no longer backported
- Messages using the old version may be rejected by updated receivers

This gives the ecosystem a minimum 24-month window to migrate — long enough for even slow-moving organizations to update.

### Migration Timeline Summary

```
Month 0          Month 12          Month 24
  │                  │                  │
  │  Phase 1:        │  Phase 2:        │  Phase 3:
  │  Dual Support    │  Deprecation     │  Sunset
  │                  │                  │
  │  Both versions   │  Old version     │  Old version
  │  work. No        │  still works     │  may be
  │  pressure to     │  but is not      │  dropped.
  │  upgrade.        │  recommended.    │
  │                  │  Security fixes  │  No more
  │                  │  backported.     │  backports.
  │                  │                  │
  ▼                  ▼                  ▼
```

-----

## Version History

### Protocol Version 1 (Current)

|Spec Revision|Date      |Key Changes                                                                                                                                                                                               |
|-------------|----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|v1.0         |March 2026|Initial specification. SHA-256 integrity, Bitcoin address identity, ECDSA signatures, block height freshness, balance gate, address chaining, application bindings.                                       |
|v1.1         |March 2026|Added ECDH encryption (AES-256-GCM), receiver pubkey fields, verify-before-download gate, ephemeral key support, 8-step verification sequence.                                                            |
|v1.2         |March 2026|Proof of Satoshi made optional (toggle + slider). Dual license (MIT OR Apache 2.0). Key discovery (`.well-known/butl.json`). Header registry. This versioning document. Comprehensive documentation suite.|

### Protocol Version 2 (Planned)

Target: 2027-2028. Expected changes:

- Key revocation mechanism (`BUTL-Revocation`)
- Multi-party signatures (`BUTL-MultiSig`, `BUTL-GroupID`)
- Post-quantum hybrid mode (`BUTL-PostQuantum`)
- Possible changes to the canonical signing payload format (which would require the version bump)

-----

## Frequently Asked Questions

**Q: Why does BUTL-Version say `1` for all of v1.0, v1.1, and v1.2?**

Because the wire format didn’t change in a backward-incompatible way. v1.1 added new fields, but they’re optional — a v1.0 receiver ignores them and still verifies correctly. v1.2 made Proof of Satoshi optional, but that’s a receiver-side configuration change, not a header change. The protocol version only increments when old receivers would break on new messages.

**Q: What happens if a receiver gets a version 2 message but only supports version 1?**

It rejects the message immediately without attempting verification. The receiver should return or log an error indicating the mismatch.

**Q: Can a sender choose to produce version 1 messages even after version 2 exists?**

Yes, during the dual-support phase. This is useful when the sender knows the receiver hasn’t upgraded yet. Version negotiation (for interactive protocols) handles this automatically.

**Q: What if an intermediary (relay server, email gateway) doesn’t understand BUTL at all?**

That’s fine. BUTL headers look like normal custom headers to a non-BUTL system. An email server just forwards the X-BUTL-* headers along with the rest of the email. A load balancer just passes the HTTP headers through. The intermediary doesn’t need to understand BUTL — only the endpoints do.

**Q: How do I know which spec revision a message was produced by?**

You don’t, and you don’t need to. The protocol version (`BUTL-Version: 1`) tells the receiver everything it needs to know about how to verify the message. The spec revision is a documentation concept, not a wire concept.

-----

*Version the protocol by what changes on the wire. Version the documentation by what changes on paper. Keep them separate and both stay clean.*