# BUTL Key Discovery

## Standard Method for Publishing and Discovering BUTL Public Keys

For BUTL encryption to work, the sender needs the receiver’s public key before the first message can be sent. This document defines a standard, interoperable method for publishing and discovering BUTL public keys using the IETF `.well-known` URI convention (RFC 8615) and an alternative DNS TXT record method.

-----

## The Problem

BUTL provides end-to-end encryption using ECDH key agreement. The sender encrypts to the receiver’s public key. But before that first message, the sender needs to obtain the receiver’s public key from somewhere.

Without a standard discovery mechanism, every application would invent its own — copy-paste, QR codes, manual exchange, proprietary directories. This kills interoperability. Two BUTL implementations that can’t discover each other’s keys can’t communicate, even though the protocol is identical.

The `.well-known/butl.json` standard solves this by giving every domain a single, predictable URL where its BUTL public key can be found.

-----

## Specification

### Endpoint

```
https://{domain}/.well-known/butl.json
```

This follows the IETF `.well-known` URI convention defined in RFC 8615, the same convention used by Let’s Encrypt (ACME), Apple (apple-app-site-association), and hundreds of other standards.

### Requirements

- The endpoint MUST be served over HTTPS. HTTP responses MUST be rejected by the client. Serving over HTTP would allow a man-in-the-middle to substitute a fake public key and decrypt all subsequent messages.
- Content-Type MUST be `application/json`.
- The response MUST be valid JSON.
- The response MUST include at minimum the `version`, `address`, and `pubkey` fields.

-----

## Response Format

### Single Identity (Most Common)

```json
{
  "version": 1,
  "address": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
  "pubkey": "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc",
  "pos_required": false,
  "pos_min_satoshis": 0,
  "freshness_window": 144,
  "supported_versions": [1],
  "enc_algos": ["AES-256-GCM"],
  "updated": "2026-03-14T00:00:00Z",
  "note": ""
}
```

### Field Definitions

|Field               |Type             |Required|Default          |Description                                                                   |
|--------------------|-----------------|--------|-----------------|------------------------------------------------------------------------------|
|`version`           |Integer          |Yes     |—                |Schema version of this JSON document. Currently `1`.                          |
|`address`           |String           |Yes     |—                |Bitcoin address (Base58Check or Bech32) of the receiver.                      |
|`pubkey`            |String           |Yes     |—                |Hex-encoded compressed secp256k1 public key (66 characters).                  |
|`pos_required`      |Boolean          |No      |`false`          |Whether this receiver requires Proof of Satoshi from senders.                 |
|`pos_min_satoshis`  |Integer		   |No		| `0` 			  |Minimum satoshis required if `pos_required` is `true`. Range: 1 — 2,100,000,000,000,000 (21,000,000 BTC). |
|`freshness_window`  |Integer          |No      |`144`            |Maximum block age accepted (in blocks). 144 blocks ≈ 24 hours.                |
|`supported_versions`|Array of Integers|No      |`[1]`            |BUTL protocol versions accepted.                                              |
|`enc_algos`         |Array of Strings |No      |`["AES-256-GCM"]`|Accepted encryption algorithms, in preference order.                          |
|`updated`           |String           |No      |—                |ISO 8601 timestamp of last key update. Helps clients detect stale caches.     |
|`note`              |String           |No      |`""`             |Human-readable note (e.g., “For API authentication, contact api@example.com”).|

-----

## Examples

### Minimal (Personal Use)

The simplest valid response. Only the three required fields.

```json
{
  "version": 1,
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "pubkey": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
}
```

A sender fetching this has everything needed to construct a BUTL message: the receiver’s address (for the header and AAD binding) and the receiver’s public key (for ECDH encryption). Default settings apply — no Proof of Satoshi required, 144-block freshness window, protocol version 1, AES-256-GCM encryption.

### API Server (Requires Proof of Satoshi)

An enterprise API that requires economic stake from callers.

```json
{
  "version": 1,
  "address": "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
  "pubkey": "03b2e1c8a4f7d9e5b6a3c2d1e0f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0",
  "pos_required": true,
  "pos_min_satoshis": 100000,
  "freshness_window": 6,
  "supported_versions": [1],
  "updated": "2026-03-14T12:00:00Z",
  "note": "Enterprise API. Proof of Satoshi required. Contact api@example.com for integration support."
}
```

The tight freshness window (6 blocks ≈ 1 hour) and high balance threshold (100,000 satoshis) indicate a high-security endpoint. Senders whose addresses hold fewer than 100,000 satoshis will be rejected at the gate.

### Multiple Identities (Organization)

An organization with different receiving identities for different purposes.

```json
{
  "version": 1,
  "keys": [
    {
      "id": "api",
      "address": "bc1qapi...",
      "pubkey": "02api...",
      "pos_required": true,
      "pos_min_satoshis": 100000,
      "freshness_window": 6,
      "note": "API authentication endpoint"
    },
    {
      "id": "support",
      "address": "bc1qsupport...",
      "pubkey": "03support...",
      "pos_required": false,
      "note": "Customer support messages"
    },
    {
      "id": "security",
      "address": "bc1qsecurity...",
      "pubkey": "02security...",
      "pos_required": false,
      "note": "Security vulnerability reports"
    }
  ],
  "updated": "2026-03-14T12:00:00Z"
}
```

When the `keys` array is present, the top-level `address` and `pubkey` fields are omitted. Each entry in the array has its own identity and configuration. Senders select the appropriate key by `id`.

Clients SHOULD present the available IDs and their notes to the user (or select programmatically based on the application context) before constructing the message.

-----

## Client Behavior

### Discovery Flow

```
1. Sender wants to send a BUTL message to someone at example.com.
2. Sender fetches https://example.com/.well-known/butl.json
3. Sender validates the response:
   a. HTTPS was used (not HTTP)
   b. JSON is valid
   c. version == 1 (or a supported schema version)
   d. address and pubkey are present
   e. pubkey is a valid 66-character hex string
   f. address correctly derives from pubkey (consistency check)
4. Sender extracts address and pubkey.
5. Sender uses pubkey for ECDH encryption.
6. Sender uses address as BUTL-Receiver and in AAD binding.
7. If pos_required is true, sender ensures their address meets the threshold.
8. Sender constructs and sends the BUTL message.
```

### Consistency Check

Clients MUST verify that the `pubkey` correctly derives to the `address`. This prevents a compromised or misconfigured server from publishing a valid-looking but mismatched address/pubkey pair. The derivation is:

```
expected_address = Base58Check(0x00 || RIPEMD-160(SHA-256(hex_decode(pubkey))))
assert expected_address == address
```

For Bech32 addresses, use the appropriate Bech32 encoding instead of Base58Check.

If the consistency check fails, the client MUST reject the response and MUST NOT send a message using the mismatched keys.

### Caching

Clients SHOULD cache the response for a reasonable period. Recommended default TTL: 1 hour. The `updated` field indicates when the key was last changed. Clients SHOULD re-fetch if their cached copy is older than 24 hours.

Caching reduces load on the server and speeds up repeated communication. However, caching too aggressively risks using a rotated key. The 1-hour default balances these concerns.

### Key Rotation

When the receiver rotates their key, they update `butl.json` with the new address and pubkey. The `updated` timestamp changes. Clients with a cached old key will construct messages encrypted to the old key. Two approaches handle this:

**Approach 1 (Recommended): Dual-key transition period.** The receiver accepts messages to both the old and new keys for a configurable transition period (recommended: 7 days). After the transition period, the old key is deactivated.

**Approach 2: Immediate rotation.** The receiver deactivates the old key immediately. Clients with stale caches will send messages that the receiver cannot decrypt. The sender receives a decryption failure response and re-fetches `butl.json` to get the new key.

Approach 1 is smoother for the sender. Approach 2 is simpler for the receiver.

### First Contact Verification

The first time a client fetches someone’s `butl.json`, it is trusting HTTPS to authenticate the server. For high-security scenarios, the public key SHOULD be verified through an out-of-band channel:

- In person (show the address on a screen, compare visually)
- Phone call (read the first and last 8 characters of the pubkey)
- Signed message on a verified social media account
- Published in a trusted directory or keyserver

After first-contact verification, subsequent fetches from the same URL are validated by the consistency of the address/pubkey pair — if the server is compromised and the attacker substitutes a different key, the address will change, which is detectable by the client.

-----

## HTTP Response Codes

|Status                 |Meaning                        |Client Action                                                                 |
|-----------------------|-------------------------------|------------------------------------------------------------------------------|
|`200 OK`               |Success                        |Parse the JSON                                                                |
|`301 / 302`            |Redirect                       |Follow the redirect (HTTPS only — reject if redirected to HTTP)               |
|`404 Not Found`        |Domain does not support BUTL   |Inform the user. Cannot send BUTL messages to this domain.                    |
|`403 Forbidden`        |Server refuses to serve the key|May be IP-restricted or access-controlled. Retry or contact the domain owner. |
|`429 Too Many Requests`|Rate limited                   |Back off and retry after the period indicated in the Retry-After header.      |
|`500 / 502 / 503`      |Server error                   |Temporary failure. Retry later. Do not treat as “domain doesn’t support BUTL.”|

-----

## DNS TXT Record Alternative

For domains that cannot serve HTTPS endpoints (e.g., email-only domains, domains without web servers, constrained environments), a DNS TXT record provides an alternative discovery method.

### Format

```
_butl.example.com  TXT  "v=1; addr=bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh; pubkey=02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc"
```

### Field Mapping

|TXT Field|JSON Equivalent   |Required             |
|---------|------------------|---------------------|
|`v`      |`version`         |Yes                  |
|`addr`   |`address`         |Yes                  |
|`pubkey` |`pubkey`          |Yes                  |
|`pos`    |`pos_required`    |No (default: `false`)|
|`pos_min`|`pos_min_satoshis`|No                   |
|`fw`     |`freshness_window`|No (default: `144`)  |

### Example with Proof of Satoshi

```
_butl.example.com  TXT  "v=1; addr=bc1q...; pubkey=02...; pos=true; pos_min=10000; fw=12"
```

### Priority

Clients SHOULD prefer `.well-known/butl.json` over DNS TXT records when both are available, because:

- JSON is richer (supports notes, multiple keys, encryption algorithm lists)
- HTTPS provides transport security (DNS queries are typically unencrypted)
- JSON is easier to parse reliably than semicolon-delimited TXT records
- `.well-known` is the established IETF standard for machine-readable metadata

DNS TXT is a fallback, not the primary mechanism.

### DNSSEC

When using the DNS TXT method, DNSSEC is RECOMMENDED for the domain. Without DNSSEC, a DNS spoofing attack could substitute a fake public key. With DNSSEC, the DNS response is cryptographically signed by the domain’s DNS operator, providing authentication comparable to HTTPS.

-----

## Security Considerations

### HTTPS Is Mandatory (for .well-known)

Serving `butl.json` over HTTP would allow any network intermediary (ISP, Wi-Fi operator, compromised router) to substitute a fake public key. The attacker’s key would be accepted by the sender, who would then encrypt messages to the attacker instead of the intended receiver. The attacker could read all messages and optionally re-encrypt them to the real receiver (a classic man-in-the-middle attack). HTTPS prevents this.

### Certificate Pinning

For high-security deployments, certificate pinning (associating the server’s TLS certificate with the expected value) provides additional protection against compromised certificate authorities. If a CA is compromised and issues a fraudulent certificate for the domain, certificate pinning detects the mismatch and rejects the connection.

### Monitoring for Unauthorized Changes

Domain owners SHOULD monitor their `butl.json` endpoint for unauthorized changes. A compromised web server could silently replace the public key with an attacker’s key. Monitoring tools can periodically fetch the endpoint and alert if the pubkey changes unexpectedly.

### Rate Limiting

Servers SHOULD rate-limit requests to `.well-known/butl.json` to prevent abuse. The endpoint returns public information, but excessive queries could be used for reconnaissance or denial of service. A rate limit of 60 requests per minute per IP is reasonable for most deployments.

### No Private Keys

The `butl.json` response MUST NEVER contain a private key. This endpoint publishes public information only: the address, the public key, and configuration preferences. If a private key is accidentally included, the identity is immediately and permanently compromised.

-----

## IANA Registration

If this convention gains sufficient adoption, it should be registered with IANA per RFC 8615:

```
URI suffix:           butl.json
Change controller:    BUTL Protocol Contributors
Specification:        This document
Related information:  https://github.com/satushinakamojo/butl-protocol
```

Until formal IANA registration, the `.well-known/butl.json` path is used by convention within the BUTL ecosystem.

-----

## Implementation Checklist

### For Servers (Publishing a Key)

- [ ] Endpoint is at `https://{domain}/.well-known/butl.json` (exact path)
- [ ] Served over HTTPS only (HTTP requests return 301 redirect to HTTPS or are refused)
- [ ] Content-Type is `application/json`
- [ ] Response includes `version`, `address`, and `pubkey`
- [ ] `pubkey` correctly derives to `address` (consistency)
- [ ] `pubkey` is a valid compressed secp256k1 point (starts with `02` or `03`, 66 hex chars)
- [ ] No private key material anywhere in the response
- [ ] Rate limiting is in place
- [ ] Monitoring for unauthorized changes is in place
- [ ] CORS headers allow cross-origin requests if browser-based clients need access

### For Clients (Fetching a Key)

- [ ] Fetch uses HTTPS only (reject HTTP, reject HTTP redirects)
- [ ] JSON is parsed safely (handle malformed responses gracefully)
- [ ] `version` is checked (reject unsupported schema versions)
- [ ] `address` and `pubkey` are both present
- [ ] Consistency check: `pubkey` derives to `address`
- [ ] `pubkey` is a valid compressed secp256k1 point
- [ ] Cache TTL is respected (default: 1 hour)
- [ ] First-contact verification is recommended to the user for high-security scenarios
- [ ] Stale cache detection: re-fetch if cached copy is older than 24 hours

-----

## Frequently Asked Questions

**Q: What if a domain doesn’t have a `.well-known/butl.json`?**

The server returns 404. The client informs the user that BUTL key discovery is not available for this domain. The sender can still communicate with the receiver if they obtain the public key through other means (direct exchange, QR code, contact book).

**Q: Can a single domain have multiple BUTL identities?**

Yes. Use the `keys` array format with distinct `id` values for each identity (see the organization example above).

**Q: How often should the key be rotated?**

There is no mandatory rotation schedule. Key rotation is recommended when: the private key may have been compromised, the key has been in use for an extended period (12+ months), or organizational security policy requires it. When rotating, use a dual-key transition period to avoid breaking cached clients.

**Q: Can I use this for email-based BUTL?**

Yes. A sender wanting to send a BUTL-encrypted email to `bob@example.com` fetches `https://example.com/.well-known/butl.json` to discover Bob’s public key. If the domain has multiple identities (the organization format), the sender selects the appropriate one.

**Q: What if the `.well-known/butl.json` and DNS TXT record contain different keys?**

The `.well-known/butl.json` takes priority. If a client fetches both and they disagree, the client SHOULD use the `.well-known` value and SHOULD alert the user or log the discrepancy.

-----

*One URL. One JSON file. The receiver’s public key, discoverable by anyone, usable by any BUTL implementation.*