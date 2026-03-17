# BUTL Python Reference Implementation

## `butl_v12.py` - Complete Library for Building BUTL Applications in Python

*From installing to sending your first BUTL-encrypted message in under 10 minutes.*

---

## What This Is

`butl_v12.py` is a single-file Python library that implements every feature of the BUTL Protocol v1.2:

- Self-sovereign identity (Bitcoin address from secp256k1 keypair)
- End-to-end encryption (ECDH + AES-256-GCM)
- Message integrity (SHA-256)
- Replay protection (Bitcoin block height freshness)
- Forward privacy (new address per message, cryptographically chained)
- Verify-before-download gate (8-step, two-phase)
- Proof of Satoshi (optional balance check, receiver-configurable toggle + slider)
- Pubkey-to-address consistency check

The library runs in two modes:

- **Production mode** - with `ecdsa` and `pycryptodome` installed. Real secp256k1 elliptic curve math. Real AES-256-GCM authenticated encryption. Suitable for production applications.
- **Stub mode** - without dependencies. Structurally identical gate logic with simplified cryptography. All 8 verification steps function correctly. Suitable for development, prototyping, and understanding the protocol.

---

## Quick Start

### Step 1: Install Python

Python 3.8 or newer is required. Check your version:

```bash
python3 --version
```

If you don't have Python 3.8+, download it from [python.org](https://www.python.org/downloads/).

### Step 2: Install Dependencies

```bash
pip install ecdsa pycryptodome
```

These are the only two dependencies:

| Package | License | What It Does |
|---------|---------|--------------|
| `ecdsa` | MIT | secp256k1 keypair generation, ECDSA signing/verification, ECDH key agreement |
| `pycryptodome` | BSD 2-Clause | AES-256-GCM authenticated encryption and decryption |

Without these, the library runs in stub mode (a warning is printed at import time).

### Step 3: Run the Demo

```bash
python3 butl_v12.py
```

This runs a complete demonstration of the protocol: genesis message, chained message, tampered payload detection, wrong receiver rejection, and Proof of Satoshi. Every step prints its verification status.

---

## Using the Library

### Import

```python
from butl_v12 import (
    BUTLKeypair,
    BUTLKeychain,
    BUTLSigner,
    BUTLGate,
    BUTLHeader,
    BUTLMessage,
    ProofOfSatoshiConfig,
    BlockchainInterface,
    GateReport,
    CheckResult,
)
```

### Create an Identity

A BUTL identity is a Bitcoin address derived from a secp256k1 keypair.

```python
# Generate a random identity
me = BUTLKeypair()

# Or from a specific seed (deterministic - same seed = same identity)
from butl_v12 import sha256
me = BUTLKeypair(sha256(b"my-secret-seed"))

# Your identity
print(f"Address:    {me.address}")           # 1A7bL3qK...
print(f"Public Key: {me.public_key_hex}")    # 02a1633c...
print(f"Private Key: {me.private_key.hex()}")  # 4f3c2a1d... - NEVER share
```

The private key is displayed here for educational purposes. In a real application, the private key is generated once, backed up immediately (ideally as a BIP-39 seed phrase), encrypted at rest, and never displayed or logged again. See [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md).

### Create a Keychain

A keychain generates a deterministic sequence of keypairs from a master seed. Each message uses a fresh keypair (address chaining).

```python
import os
from butl_v12 import BUTLKeychain

# Random seed (back this up!)
keychain = BUTLKeychain(seed=os.urandom(32))

# First keypair (genesis)
kp0 = keychain.next_keypair()
print(f"Message 0: {kp0.address}")

# Second keypair (chained to first)
kp1 = keychain.next_keypair()
print(f"Message 1: {kp1.address}")

# Thread identity (constant across all messages)
print(f"Thread ID: {keychain.thread_id}")
```

### Send an Encrypted Message

```python
from butl_v12 import BUTLKeychain, BUTLSigner, sha256

# Sender setup
sender_seed = sha256(b"sender-secret-seed")
sender_keychain = BUTLKeychain(seed=sender_seed)
signer = BUTLSigner(sender_keychain)

# Receiver's public identity (obtained via .well-known/butl.json,
# contact exchange, QR code, or any other method)
receiver_pubkey = receiver.public_key      # 33 bytes
receiver_address = receiver.address         # string

# Sign, encrypt, and package the message
msg = signer.sign_and_encrypt(
    body=b"Hello from BUTL!",
    receiver_pubkey=receiver_pubkey,
    receiver_address=receiver_address,
)

# msg.header         - BUTL header (cleartext, for Phase 1)
# msg.encrypted_payload - encrypted body (for Phase 2)
```

### Receive and Verify a Message

```python
from butl_v12 import BUTLKeypair, BUTLGate, ProofOfSatoshiConfig, sha256

# Receiver setup
receiver = BUTLKeypair(sha256(b"receiver-secret-seed"))
gate = BUTLGate(
    receiver_keypair=receiver,
    pos_config=ProofOfSatoshiConfig(enabled=False),
    freshness_window=144,  # ~24 hours
)

# Phase 1: Verify header only (no payload on device yet)
report = gate.check_header(msg.header)

if report.gate_passed:
    # Phase 2: Download and decrypt payload
    plaintext, report = gate.accept_payload(
        msg.header, msg.encrypted_payload, report
    )

    if report.fully_verified:
        print(f"Message verified and decrypted: {plaintext}")
    else:
        print(f"Payload verification failed")
        print(report.summary())
else:
    print(f"Gate CLOSED - message rejected")
    print(report.summary())
```

### Chained Messages (Conversation)

```python
# Sender sends message 1 (genesis)
msg1 = signer.sign_and_encrypt(body1, rx_pubkey, rx_address)
sender1_pk = bytes.fromhex(msg1.header.sender_pubkey)

# Receiver verifies message 1
report1 = gate.check_header(msg1.header)
pt1, report1 = gate.accept_payload(msg1.header, msg1.encrypted_payload, report1)

# Sender sends message 2 (chained)
msg2 = signer.sign_and_encrypt(body2, rx_pubkey, rx_address)

# Receiver verifies message 2 - passes previous sender's pubkey for chain proof
report2 = gate.check_header(msg2.header, prev_pubkey=sender1_pk)
pt2, report2 = gate.accept_payload(msg2.header, msg2.encrypted_payload, report2)
```

The `prev_pubkey` parameter tells the gate which public key to verify the chain proof against. For genesis messages (no `prev_addr`), it can be omitted.

### Proof of Satoshi (Optional)

```python
from butl_v12 import ProofOfSatoshiConfig

# Disabled (default) - pure math, no blockchain query
gate_no_pos = BUTLGate(
    receiver,
    pos_config=ProofOfSatoshiConfig(enabled=False),
)

# Enabled with threshold - requires sender to hold >= 10,000 satoshis
gate_with_pos = BUTLGate(
    receiver,
    pos_config=ProofOfSatoshiConfig(enabled=True, min_satoshis=10000),
)

# Enabled with high threshold - enterprise use
gate_enterprise = BUTLGate(
    receiver,
    pos_config=ProofOfSatoshiConfig(enabled=True, min_satoshis=100000),
)
```

When Proof of Satoshi is disabled, Step 5 is skipped and the protocol operates as pure math with zero external dependencies. When enabled, Step 5 queries the blockchain to verify the sender's balance meets the threshold.

### Ephemeral Keys (Enhanced Forward Secrecy)

```python
# Use a separate key for encryption (not the signing key)
msg = signer.sign_and_encrypt(
    body=b"Extra secure message",
    receiver_pubkey=receiver.public_key,
    receiver_address=receiver.address,
    use_ephemeral=True,
)
# BUTL-EphemeralPubKey is automatically included in the header
# Receiver automatically uses it for ECDH instead of SenderPubKey
```

### Header Serialization

```python
# To text headers (email X-headers / HTTP headers)
text = msg.header.to_text_headers()
print(text)
# BUTL-Version: 1
# BUTL-Sender: 1A7bL3qK...
# BUTL-SenderPubKey: 02a1633c...
# ...

# To JSON
json_str = msg.header.to_json()

# To dictionary
d = msg.header.to_dict()

# Parse from text headers
header = BUTLHeader.from_text_headers(text)

# Parse from JSON
header = BUTLHeader.from_json(json_str)

# Parse from dictionary
header = BUTLHeader.from_dict(d)
```

### Reading the Gate Report

```python
report = gate.check_header(header)

# Individual step results
print(report.structural.value)      # "PASS", "FAIL", or "SKIP"
print(report.receiver_match.value)
print(report.signature.value)
print(report.freshness.value)
print(report.pos.value)
print(report.chain.value)
print(report.payload_hash.value)
print(report.decryption.value)

# Aggregate results
print(report.gate_passed)       # True if steps 1-6 all passed
print(report.fully_verified)    # True if all 8 steps passed

# Human-readable summary
print(report.summary())

# Error details
for error in report.errors:
    print(f"Error: {error}")
```

### Custom Blockchain Provider

For production, replace the stub blockchain with a real provider:

```python
from butl_v12 import BlockchainInterface
import requests

class MempoolBlockchain(BlockchainInterface):
    """Real blockchain provider using mempool.space API."""

    def get_current_block(self):
        r = requests.get("https://mempool.space/api/blocks/tip/height")
        height = int(r.text)
        r2 = requests.get(f"https://mempool.space/api/block-height/{height}")
        block_hash = r2.text
        return (height, block_hash)

    def verify_block(self, height, block_hash):
        r = requests.get(f"https://mempool.space/api/block-height/{height}")
        return r.text == block_hash

    def get_chain_height(self):
        r = requests.get("https://mempool.space/api/blocks/tip/height")
        return int(r.text)

    def check_balance(self, address, min_satoshis=1):
        r = requests.get(f"https://mempool.space/api/address/{address}")
        data = r.json()
        funded = data["chain_stats"]["funded_txo_sum"]
        spent = data["chain_stats"]["spent_txo_sum"]
        balance = funded - spent
        return balance >= min_satoshis


# Use it
blockchain = MempoolBlockchain()
signer = BUTLSigner(keychain, blockchain=blockchain)
gate = BUTLGate(receiver, blockchain=blockchain)
```

---

## API Reference

### Classes

| Class | Purpose |
|-------|---------|
| `BUTLKeypair` | Single secp256k1 keypair. Identity, signing, ECDH, address derivation. |
| `BUTLKeychain` | Deterministic key sequence from master seed. Address chaining. |
| `BUTLSigner` | Sender-side: sign, encrypt, and package complete BUTL messages. |
| `BUTLGate` | Receiver-side: 8-step verify-before-download gate. Two-phase. |
| `BUTLHeader` | Protocol header. Serialize to/from text, JSON, dict. Canonical payload. |
| `BUTLMessage` | Header + encrypted payload. The complete transmissible unit. |
| `ProofOfSatoshiConfig` | Toggle + slider for balance check. Receiver configuration. |
| `BlockchainInterface` | Abstract blockchain provider. Stub included, real implementation required for production. |
| `GateReport` | Detailed results of all 8 verification steps. |
| `CheckResult` | Enum: `PASS`, `FAIL`, `SKIP`. |

### Functions

| Function | Purpose |
|----------|---------|
| `sha256(data)` | SHA-256 hash -> 32 bytes |
| `sha256_hex(data)` | SHA-256 hash -> 64 hex chars |
| `double_sha256(data)` | Bitcoin-style double SHA-256 |
| `public_key_to_address(pubkey)` | Compressed pubkey (33 bytes) -> Bitcoin P2PKH address |
| `verify_address_pubkey_consistency(addr, pubkey_hex)` | Verify pubkey derives to address -> bool |
| `encrypt_payload(plaintext, secret, sender, receiver)` | AES-256-GCM encrypt with AAD |
| `decrypt_payload(encrypted, secret, sender, receiver)` | AES-256-GCM decrypt with AAD |
| `base58check_encode(version, payload)` | Base58Check encoding |

### Module Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `__version__` | `"1.2.0"` | Library version |
| `__protocol_version__` | `1` | BUTL protocol version (the wire format version) |
| `STUB_MODE` | `True/False` | Whether dependencies are missing |
| `HAS_ECDSA` | `True/False` | Whether `ecdsa` is installed |
| `HAS_AES` | `True/False` | Whether `pycryptodome` is installed |

---

## File Structure

```
python/
|-- butl_v12.py      Reference implementation (this file)
+-- README.md        This document
```

A single file. No package structure. No `setup.py`. No `__init__.py`. Copy `butl_v12.py` into your project and import it. That's it.

For production deployment as a package (planned for v1.3):

```bash
pip install butl       # Not yet available - planned for PyPI
```

---

## Integration Examples

### Passwordless Website Login

```python
# Server presents a challenge
challenge = {
    "nonce": os.urandom(16).hex(),
    "block_height": blockchain.get_chain_height(),
    "server_address": server_keypair.address,
}

# Client signs the challenge with BUTL
msg = signer.sign_and_encrypt(
    body=json.dumps(challenge).encode(),
    receiver_pubkey=server_keypair.public_key,
    receiver_address=server_keypair.address,
)

# Server verifies - if gate passes, the user is authenticated
report = gate.check_header(msg.header)
if report.gate_passed:
    pt, report = gate.accept_payload(msg.header, msg.encrypted_payload, report)
    if report.fully_verified:
        user_address = msg.header.sender
        # Authenticated as user_address - no password needed
```

### Authenticated API Request

```python
# Client sends an authenticated request
msg = signer.sign_and_encrypt(
    body=json.dumps({"action": "get_balance", "account": "12345"}).encode(),
    receiver_pubkey=api_server.public_key,
    receiver_address=api_server.address,
)

# Transmit as HTTP headers + body
headers = {}
for line in msg.header.to_text_headers().split("\n"):
    key, value = line.split(": ", 1)
    headers[key] = value
response = requests.post(
    "https://api.example.com/v1/data",
    headers=headers,
    data=msg.encrypted_payload,
)
```

### Encrypted File Transfer

```python
# Read and encrypt a file
with open("document.pdf", "rb") as f:
    file_data = f.read()

msg = signer.sign_and_encrypt(
    body=file_data,
    receiver_pubkey=recipient.public_key,
    receiver_address=recipient.address,
)

# Recipient verifies before saving to disk
report = gate.check_header(msg.header)
if report.gate_passed:
    plaintext, report = gate.accept_payload(
        msg.header, msg.encrypted_payload, report
    )
    if report.fully_verified:
        with open("document_received.pdf", "wb") as f:
            f.write(plaintext)
```

---

## Security Notes

- **Private keys:** Never log, print, or transmit private keys. Use `keypair.secure_delete()` after chain proof creation to zero the memory.
- **Stub mode:** Not safe for production. The stub uses simplified cryptography that is structurally correct but not cryptographically secure. Always install `ecdsa` and `pycryptodome` for production.
- **Blockchain stub:** The default `BlockchainInterface` returns hardcoded values. Production deployments must use a real blockchain provider (see the custom provider example above).
- **Key storage:** This library does not handle key storage. Private keys must be encrypted at rest using OS keychains, HSMs, or password-derived encryption (Argon2id + AES-256-GCM). See [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md).
- **Timing attacks:** The `ecdsa` library's verification is not guaranteed constant-time. For maximum security, use bindings to `libsecp256k1` (the C library used by Bitcoin Core).

---

## Troubleshooting

### "RuntimeWarning: BUTL running in STUB MODE"

Install the dependencies:

```bash
pip install ecdsa pycryptodome
```

If you're using a virtual environment, make sure it's activated before installing.

### "ModuleNotFoundError: No module named 'Crypto'"

Some systems install pycryptodome as `Cryptodome` instead of `Crypto`. The library checks for both. If neither works:

```bash
pip uninstall pycrypto pycryptodome
pip install pycryptodome
```

### "ecdsa.keys.BadSignatureError"

The signature is invalid. This means one of the following: the canonical signing payload was constructed differently by sender and receiver (field order or formatting mismatch), the header was tampered with in transit, or the wrong public key was used for verification.

### "AES-GCM authentication failed"

The decryption key (ECDH shared secret) is wrong, or the encrypted payload was tampered with. This is expected when: the wrong receiver tries to decrypt (different private key = different shared secret), the payload was modified in transit (GCM tag check fails), or the AAD doesn't match (addresses changed).

---

## Related Documents

| Document | Description |
|----------|-------------|
| [BUTL_Protocol_Specification_v1.2.md](../spec/BUTL_Protocol_Specification_v1.2.md) | Complete protocol specification |
| [HEADER_REGISTRY.md](../spec/HEADER_REGISTRY.md) | All BUTL header fields defined |
| [TEST_VECTORS.md](../spec/TEST_VECTORS.md) | Deterministic test cases for interoperability |
| [KEY_IS_IDENTITY_WARNING.md](../KEY_IS_IDENTITY_WARNING.md) | Critical key safety information |
| [butl_mvp_v12.py](../mvp/butl_mvp_v12.py) | Zero-dependency MVP proving all 8 claims |
| [SECURITY.md](../SECURITY.md) | Security policy and known limitations |

---

*One file. Two dependencies. Eight-step verified trust. Get started in under 10 minutes.*
