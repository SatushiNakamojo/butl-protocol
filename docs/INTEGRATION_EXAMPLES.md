# BUTL Integration Examples

## Code Snippets for Real-World Applications

Copy-paste examples showing how to integrate BUTL into passwordless login, authenticated email, REST API authentication, IoT device communication, encrypted file transfer, and document signing. Each example includes both the sender and receiver side.

All examples use the Python reference implementation (`butl_v12.py`). The same patterns apply to the Rust implementation with equivalent API calls.

---

## 1. Passwordless Website Login

Replace passwords with BUTL signatures. The server presents a challenge. The client signs it. The server verifies. No password database. No OAuth. No third party.

### Server Side (Challenge Generation)

```python
import json
import os
import time
from butl_v12 import BUTLKeypair, BUTLGate, ProofOfSatoshiConfig, sha256, sha256_hex

# Server's BUTL identity (generated once, stored securely)
server = BUTLKeypair(sha256(b"server-secret-seed"))

def generate_challenge():
    """Create a login challenge for the client."""
    challenge = {
        "server_address": server.address,
        "server_pubkey": server.public_key_hex,
        "nonce": os.urandom(16).hex(),
        "block_height": 890412,  # Use real block height in production
        "timestamp": int(time.time()),
        "expires": int(time.time()) + 300,  # 5 minutes
    }
    return json.dumps(challenge)

# Send challenge to client via HTTPS
challenge_json = generate_challenge()
print(f"Challenge: {challenge_json}")
```

### Client Side (Sign the Challenge)

```python
from butl_v12 import BUTLKeychain, BUTLSigner, sha256

# Client's BUTL identity
client_keychain = BUTLKeychain(seed=sha256(b"client-secret-seed"))
signer = BUTLSigner(client_keychain)

# Receive challenge from server, sign and encrypt it
challenge_bytes = challenge_json.encode("utf-8")

msg = signer.sign_and_encrypt(
    body=challenge_bytes,
    receiver_pubkey=server.public_key,
    receiver_address=server.address,
)

# Send msg.header (as HTTP headers) and msg.encrypted_payload (as body)
# to the server's /login endpoint
login_headers = msg.header.to_text_headers()
login_body = msg.encrypted_payload
```

### Server Side (Verify and Authenticate)

```python
from butl_v12 import BUTLGate, BUTLHeader, ProofOfSatoshiConfig

gate = BUTLGate(
    receiver_keypair=server,
    pos_config=ProofOfSatoshiConfig(enabled=False),
    freshness_window=144,
)

# Parse the received header
header = BUTLHeader.from_text_headers(login_headers)

# Phase 1: Verify header
report = gate.check_header(header)

if report.gate_passed:
    # Phase 2: Decrypt payload
    plaintext, report = gate.accept_payload(header, login_body, report)

    if report.fully_verified:
        # Verify the challenge contents
        challenge = json.loads(plaintext)
        if challenge["server_address"] == server.address:
            if challenge["expires"] > time.time():
                user_address = header.sender
                print(f"Authenticated: {user_address}")
                # Create session for user_address - no password needed
            else:
                print("Challenge expired")
        else:
            print("Challenge mismatch")
    else:
        print(f"Payload verification failed: {report.summary()}")
else:
    print(f"Gate closed: {report.summary()}")
```

### What the User Experiences

1. Visit `https://example.com/login`
2. Browser extension (BUTL-Login) detects the challenge
3. User clicks "Sign In with BUTL"
4. Authenticated. No password typed. No email entered. No 2FA code.

---

## 2. Authenticated Email (SMTP with BUTL Headers)

Add BUTL verification to email without changing the email protocol. BUTL fields ride as X-headers. The encrypted body is a MIME attachment. The recipient's email client verifies before rendering.

### Sender Side (Compose and Sign)

```python
import email.mime.multipart
import email.mime.text
import email.mime.base
import smtplib
import base64
from butl_v12 import BUTLKeychain, BUTLSigner, sha256

# Sender's identity
sender_keychain = BUTLKeychain(seed=sha256(b"alice-email-seed"))
signer = BUTLSigner(sender_keychain)

# Recipient's BUTL public key (from .well-known/butl.json or contact book)
recipient_pubkey = recipient.public_key
recipient_address = recipient.address

# The email body
email_body = b"Hi Bob, the quarterly report is attached. - Alice"

# Sign and encrypt with BUTL
msg = signer.sign_and_encrypt(
    body=email_body,
    receiver_pubkey=recipient_pubkey,
    receiver_address=recipient_address,
)

# Build the email
email_msg = email.mime.multipart.MIMEMultipart()
email_msg["From"] = "alice@example.com"
email_msg["To"] = "bob@example.com"
email_msg["Subject"] = "Quarterly Report (BUTL-Encrypted)"

# Add BUTL headers as X-headers
for line in msg.header.to_text_headers().split("\n"):
    key, value = line.split(": ", 1)
    email_msg[f"X-{key}"] = value

# Attach encrypted payload
payload_part = email.mime.base.MIMEBase("application", "octet-stream")
payload_part.set_payload(base64.b64encode(msg.encrypted_payload))
payload_part.add_header("Content-Disposition", "attachment",
                        filename="butl-payload.enc")
payload_part.add_header("Content-Transfer-Encoding", "base64")
email_msg.attach(payload_part)

# Send via SMTP (standard email infrastructure)
# with smtplib.SMTP("smtp.example.com", 587) as smtp:
#     smtp.starttls()
#     smtp.login("alice@example.com", "email-password")
#     smtp.send_message(email_msg)

print("Email composed with BUTL headers:")
print(email_msg.as_string()[:500] + "...")
```

### Recipient Side (Verify Before Rendering)

```python
import email
import base64
from butl_v12 import BUTLKeypair, BUTLGate, BUTLHeader, ProofOfSatoshiConfig, sha256

# Recipient's identity
recipient = BUTLKeypair(sha256(b"bob-email-seed"))
gate = BUTLGate(recipient, pos_config=ProofOfSatoshiConfig(enabled=False))

# Parse the received email
# raw_email = ... (received via IMAP/POP3)
# parsed = email.message_from_string(raw_email)

# Extract BUTL headers from X-headers
butl_header_lines = []
for key in parsed.keys():
    if key.startswith("X-BUTL-"):
        clean_key = key[2:]  # Remove "X-" prefix
        butl_header_lines.append(f"{clean_key}: {parsed[key]}")

header = BUTLHeader.from_text_headers("\n".join(butl_header_lines))

# Extract encrypted payload from attachment
for part in parsed.walk():
    if part.get_filename() == "butl-payload.enc":
        encrypted_payload = base64.b64decode(part.get_payload())
        break

# Verify before rendering
report = gate.check_header(header)

if report.gate_passed:
    plaintext, report = gate.accept_payload(header, encrypted_payload, report)
    if report.fully_verified:
        print(f"Verified sender: {header.sender}")
        print(f"Message: {plaintext.decode('utf-8')}")
        # Safe to render - verified and decrypted
    else:
        print("Payload verification failed - do not render")
else:
    print("BUTL verification failed - possible phishing")
    # Do not render the email. Do not download attachments.
```

### What This Prevents

- **Phishing:** A forged email claiming to be from `alice@example.com` will fail BUTL signature verification because the attacker doesn't have Alice's private key.
- **Tampering:** If the email body is modified in transit, the payload hash won't match.
- **Impersonation:** The sender's Bitcoin address is cryptographically bound to the signature. No one can impersonate Alice without her private key.

---

## 3. REST API Authentication

Replace API keys with BUTL signatures. Every request is signed, encrypted, and verified. No API key to steal. No token to expire. No OAuth dance.

### Client Side (Send Authenticated Request)

```python
import requests
from butl_v12 import BUTLKeychain, BUTLSigner, sha256

# Client's identity
client_keychain = BUTLKeychain(seed=sha256(b"api-client-seed"))
signer = BUTLSigner(client_keychain)

# API server's public key (from .well-known/butl.json)
api_pubkey = api_server.public_key
api_address = api_server.address

# The API request body
request_body = b'{"action": "transfer", "amount": 1000, "to": "account_456"}'

# Sign and encrypt
msg = signer.sign_and_encrypt(
    body=request_body,
    receiver_pubkey=api_pubkey,
    receiver_address=api_address,
)

# Build HTTP request with BUTL headers
headers = {}
for line in msg.header.to_text_headers().split("\n"):
    key, value = line.split(": ", 1)
    headers[key] = value

response = requests.post(
    "https://api.example.com/v1/transactions",
    headers=headers,
    data=msg.encrypted_payload,
)

print(f"Response: {response.status_code}")
```

### Server Side (Verify and Process)

```python
from flask import Flask, request, jsonify
from butl_v12 import BUTLKeypair, BUTLGate, BUTLHeader, ProofOfSatoshiConfig, sha256

app = Flask(__name__)

# Server's identity
server = BUTLKeypair(sha256(b"api-server-seed"))
gate = BUTLGate(
    server,
    pos_config=ProofOfSatoshiConfig(enabled=True, min_satoshis=10000),
    freshness_window=6,  # ~1 hour - tight for API security
)

# Authorized clients (address -> permissions)
authorized = {
    "1ClientAddress...": ["transfer", "balance", "history"],
}

@app.route("/v1/transactions", methods=["POST"])
def handle_transaction():
    # Extract BUTL header from HTTP headers
    butl_lines = []
    for key, value in request.headers:
        if key.startswith("BUTL-"):
            butl_lines.append(f"{key}: {value}")

    if not butl_lines:
        return jsonify({"error": "Missing BUTL headers"}), 401

    header = BUTLHeader.from_text_headers("\n".join(butl_lines))

    # Phase 1: Verify header (no payload processed yet)
    report = gate.check_header(header)

    if not report.gate_passed:
        return jsonify({
            "error": "BUTL verification failed",
            "details": report.errors,
        }), 403

    # Check authorization
    if header.sender not in authorized:
        return jsonify({"error": "Unknown client"}), 403

    # Phase 2: Decrypt payload
    encrypted_payload = request.get_data()
    plaintext, report = gate.accept_payload(header, encrypted_payload, report)

    if not report.fully_verified:
        return jsonify({"error": "Payload verification failed"}), 400

    # Process the request
    import json
    body = json.loads(plaintext)
    action = body.get("action")

    if action not in authorized[header.sender]:
        return jsonify({"error": "Unauthorized action"}), 403

    # Execute the action...
    return jsonify({
        "status": "success",
        "authenticated_as": header.sender,
        "action": action,
    })
```

### Why This Is Better Than API Keys

| Property | API Keys | BUTL Authentication |
|----------|----------|-------------------|
| Stolen key = full access | Yes | No (key is never transmitted) |
| Man-in-the-middle | Possible (if key intercepted) | Impossible (ECDSA signature) |
| Replay attack | Possible (key reuse) | Blocked (block height freshness) |
| Request tampering | Undetected | Detected (signature covers headers) |
| Payload encryption | Separate (TLS only) | Built-in (ECDH + AES-GCM) |
| Payload integrity | None | SHA-256 hash + GCM tag |
| Identity verification | Shared secret | Cryptographic proof |
| Revocation | Change key (all clients affected) | Per-client (address-based) |

---

## 4. IoT Device Authentication

Every IoT device gets a BUTL identity. Device-to-server and device-to-device communication is signed, encrypted, and verified. Firmware updates are BUTL-signed - the device verifies before installing.

### Device Side (Register and Authenticate)

```python
from butl_v12 import BUTLKeypair, BUTLKeychain, BUTLSigner, sha256
import json

# Each device generates a unique identity at manufacture
# The seed is derived from hardware-unique data (serial number, etc.)
device_seed = sha256(b"device-serial-ABC123-secret")
device_keychain = BUTLKeychain(seed=device_seed)
signer = BUTLSigner(device_keychain)

# Server's public key (provisioned during manufacturing)
server_pubkey = server.public_key
server_address = server.address

# Registration message
registration = json.dumps({
    "device_type": "temperature_sensor",
    "serial": "ABC123",
    "firmware_version": "2.1.0",
    "capabilities": ["temperature", "humidity"],
}).encode("utf-8")

msg = signer.sign_and_encrypt(
    body=registration,
    receiver_pubkey=server_pubkey,
    receiver_address=server_address,
)

# Send to server via MQTT, HTTP, TCP, or any transport
# transport.send(msg.header.to_json(), msg.encrypted_payload)
print(f"Device registered: {device_keychain.current.address}")
```

### Device Side (Send Sensor Data)

```python
import json
import time

# Periodic sensor reading
reading = json.dumps({
    "temperature_c": 22.5,
    "humidity_pct": 45.2,
    "timestamp": int(time.time()),
    "battery_pct": 87,
}).encode("utf-8")

# Sign and encrypt - uses next address in chain (automatic key rotation)
msg = signer.sign_and_encrypt(
    body=reading,
    receiver_pubkey=server_pubkey,
    receiver_address=server_address,
)

# Each reading uses a fresh address, chained to the previous
# The server verifies the chain proof to confirm it's the same device
print(f"Sensor reading sent from: {device_keychain.current.address}")
print(f"  Chain index: {device_keychain.sequence_number}")
print(f"  Thread ID:   {device_keychain.thread_id}")
```

### Server Side (Verify Device Identity)

```python
from butl_v12 import BUTLGate, BUTLHeader, ProofOfSatoshiConfig

# Server setup
gate = BUTLGate(
    server_keypair,
    pos_config=ProofOfSatoshiConfig(enabled=False),  # IoT devices don't hold BTC
    freshness_window=144,
)

# Track registered devices
devices = {}  # thread_id -> {serial, last_pubkey, last_seq}

def handle_device_message(header_json, encrypted_payload):
    header = BUTLHeader.from_json(header_json)

    # Look up previous sender pubkey for chain proof verification
    prev_pk = None
    if header.thread_id in devices:
        prev_pk = bytes.fromhex(devices[header.thread_id]["last_pubkey"])

    # Verify
    report = gate.check_header(header, prev_pubkey=prev_pk)

    if report.gate_passed:
        plaintext, report = gate.accept_payload(
            header, encrypted_payload, report
        )
        if report.fully_verified:
            data = json.loads(plaintext)
            # Update device tracking
            devices[header.thread_id] = {
                "serial": data.get("serial", "unknown"),
                "last_pubkey": header.sender_pubkey,
                "last_seq": header.seq_num,
            }
            print(f"Verified device {header.thread_id[:16]}...: {data}")
            return data
    return None
```

### Firmware Update Verification

```python
def verify_firmware_update(header, firmware_binary):
    """Device-side firmware update verification.
    The manufacturer signs the update with their BUTL key.
    The device verifies before installing."""

    report = gate.check_header(header)

    if not report.gate_passed:
        print("Firmware update REJECTED - signature verification failed")
        print("Keeping current firmware.")
        return False

    plaintext, report = gate.accept_payload(header, firmware_binary, report)

    if report.fully_verified:
        print(f"Firmware update VERIFIED from: {header.sender}")
        # Safe to install
        return True
    else:
        print("Firmware update REJECTED - payload tampering detected")
        return False
```

---

## 5. Encrypted File Transfer

Send files with end-to-end encryption and verify-before-download protection. The recipient verifies the sender's identity and the file's integrity before the file touches their disk.

### Sender Side

```python
import os
from butl_v12 import BUTLKeychain, BUTLSigner, sha256

sender_keychain = BUTLKeychain(seed=sha256(b"sender-file-seed"))
signer = BUTLSigner(sender_keychain)

# Read the file
file_path = "quarterly_report.pdf"
with open(file_path, "rb") as f:
    file_data = f.read()

print(f"Sending: {file_path} ({len(file_data)} bytes)")

# Sign and encrypt the entire file
msg = signer.sign_and_encrypt(
    body=file_data,
    receiver_pubkey=recipient.public_key,
    receiver_address=recipient.address,
)

# Transmit header and payload separately (two-phase delivery)
# Phase 1: Send header
# transport.send_header(msg.header.to_json())

# Phase 2: Send payload only after recipient confirms gate passed
# transport.send_payload(msg.encrypted_payload)

print(f"File encrypted: {len(msg.encrypted_payload)} bytes")
print(f"Payload hash:   {msg.header.payload_hash[:32]}...")
```

### Recipient Side

```python
from butl_v12 import BUTLKeypair, BUTLGate, BUTLHeader, ProofOfSatoshiConfig, sha256

recipient = BUTLKeypair(sha256(b"recipient-file-seed"))
gate = BUTLGate(recipient, pos_config=ProofOfSatoshiConfig(enabled=False))

# Phase 1: Receive and verify header (no file data on disk yet)
# header_json = transport.receive_header()
header = BUTLHeader.from_json(header_json)

# Optional: Check payload size before downloading
if header.payload_size > 100_000_000:  # 100 MB limit
    print(f"File too large: {header.payload_size} bytes. Rejecting.")
    # transport.reject()
else:
    report = gate.check_header(header)

    if report.gate_passed:
        print(f"Gate OPEN - downloading file from verified sender: {header.sender}")
        # Phase 2: Download the encrypted payload
        # encrypted_payload = transport.receive_payload()

        plaintext, report = gate.accept_payload(
            header, encrypted_payload, report
        )

        if report.fully_verified:
            # Safe to save - sender verified, file integrity confirmed
            output_path = "quarterly_report_received.pdf"
            with open(output_path, "wb") as f:
                f.write(plaintext)
            print(f"File saved: {output_path} ({len(plaintext)} bytes)")
            print(f"Verified from: {header.sender}")
        else:
            print("File verification failed - not saved")
            # plaintext is already None, nothing to save
    else:
        print("Gate CLOSED - file rejected before download")
        print(f"Reason: {report.errors}")
        # Phase 2 never happens. The file never touches the disk.
```

### What This Prevents

- **Malware delivery:** A malicious file from an unverified sender is blocked at the gate. The file never reaches the disk.
- **Man-in-the-middle:** An attacker who intercepts the transfer cannot decrypt the file (ECDH) or forge the signature (ECDSA).
- **Tampering:** If the file is modified in transit, the payload hash check (Step 7) or GCM authentication (Step 8) catches it.
- **Resource exhaustion:** The `BUTL-PayloadSize` field lets the recipient reject oversized files before downloading.

---

## 6. Document Signing with Timestamp

Sign a document with a BUTL identity and anchor the signature to a specific Bitcoin block height, creating a verifiable record that the document existed and was signed at that point in time.

### Signing a Document

```python
import json
import time
from butl_v12 import BUTLKeypair, sha256, sha256_hex

# Signer's identity
signer_kp = BUTLKeypair(sha256(b"notary-seed"))

# The document to sign
with open("contract.pdf", "rb") as f:
    document = f.read()

doc_hash = sha256_hex(document)
block_height = 890412  # Current Bitcoin block height

# Build the signing record
record = {
    "document_hash": doc_hash,
    "signer_address": signer_kp.address,
    "signer_pubkey": signer_kp.public_key_hex,
    "block_height": block_height,
    "timestamp": int(time.time()),
    "statement": "I, the holder of the above address, attest to having "
                 "reviewed and signed the document with the above hash.",
}

# Sign the record
record_bytes = json.dumps(record, sort_keys=True).encode("utf-8")
record_hash = sha256(record_bytes)
signature = signer_kp.sign(record_hash)

# The signature record
import base64
signed_record = {
    **record,
    "signature": base64.b64encode(signature).decode("ascii"),
}

# Save the signed record alongside the document
with open("contract_signature.json", "w") as f:
    json.dump(signed_record, f, indent=2)

print(f"Document signed by: {signer_kp.address}")
print(f"Document hash:      {doc_hash[:32]}...")
print(f"Block height:       {block_height}")
print(f"Signature record saved to: contract_signature.json")
```

### Verifying a Signed Document

```python
import json
import base64
from butl_v12 import BUTLKeypair, sha256, sha256_hex, verify_address_pubkey_consistency

# Load the signature record
with open("contract_signature.json", "r") as f:
    record = json.load(f)

# Load the document
with open("contract.pdf", "rb") as f:
    document = f.read()

# Step 1: Verify the document hash
computed_hash = sha256_hex(document)
if computed_hash != record["document_hash"]:
    print("FAILED: Document has been modified since signing")
else:
    print("Document hash: MATCH")

# Step 2: Verify pubkey-to-address consistency
if not verify_address_pubkey_consistency(
    record["signer_address"], record["signer_pubkey"]
):
    print("FAILED: Public key does not derive to claimed address")
else:
    print("Consistency check: PASS")

# Step 3: Verify the signature
signature = base64.b64decode(record["signature"])
record_copy = {k: v for k, v in record.items() if k != "signature"}
record_bytes = json.dumps(record_copy, sort_keys=True).encode("utf-8")
record_hash = sha256(record_bytes)
pubkey = bytes.fromhex(record["signer_pubkey"])

valid = BUTLKeypair.verify(
    record["signer_address"], signature, record_hash, pubkey
)

if valid:
    print(f"Signature: VALID")
    print(f"Signed by: {record['signer_address']}")
    print(f"At block:  {record['block_height']}")
    print(f"Document is authentic and unmodified.")
else:
    print("FAILED: Signature is invalid")
```

---

## Integration Pattern Summary

Every BUTL integration follows the same pattern regardless of the application domain:

```
SENDER                                  RECEIVER

1. Construct the body                   1. Receive the header (Phase 1)
2. Call signer.sign_and_encrypt()       2. Run gate.check_header()
3. Transmit header (Phase 1)            3. If gate passes: request payload
4. Wait for receiver confirmation       4. Receive payload (Phase 2)
5. Transmit payload (Phase 2)           5. Run gate.accept_payload()
                                        6. If fully verified: process plaintext
                                        7. If failed: discard everything
```

The body can be a chat message, an email, an API request, a sensor reading, a firmware binary, a file, or a signed document. The pattern is the same. The verification is the same. The trust is the same.

---

*Six use cases. One protocol. Same pattern everywhere. Adapt these examples to your application.*
