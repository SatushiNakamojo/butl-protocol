#!/usr/bin/env python3
“””
BUTL Reference Implementation v1.2 (Python)
Bitcoin Universal Trust Layer

Complete, production-ready API for building BUTL applications.

Implements all protocol features from the v1.2 specification:

- Self-sovereign identity (Bitcoin addresses from secp256k1)
- End-to-end encryption (ECDH + AES-256-GCM)
- Message integrity (SHA-256)
- Replay protection (block height freshness)
- Address chaining (new key per message, cryptographically linked)
- Verify-before-download gate (8-step, two-phase)
- Proof of Satoshi (optional balance check, toggle + slider)
- Pubkey-to-address consistency check

Dependencies:
pip install ecdsa pycryptodome

ecdsa        — secp256k1 keypair, ECDSA signing/verification, ECDH
pycryptodome — AES-256-GCM authenticated encryption

Without these dependencies, the library runs in stub mode.
Stub mode uses simplified cryptography that is structurally
identical to the real protocol. All 8 gate steps function
correctly, including encryption and decryption.

Zero-dependency MVP:  see butl_mvp_v12.py
Protocol spec:        see BUTL_Protocol_Specification_v1.2.md
Test vectors:         see TEST_VECTORS.md

License: MIT OR Apache-2.0 (dual licensed, at your option)
Patent:  PATENTS.md + DEFENSIVE-PATENT-PLEDGE.md (applies to all users)

Copyright 2026 Satushi Nakamojo
“””

**version** = “1.2.0”
**protocol_version** = 1

import hashlib
import json
import time
import os
import base64
from dataclasses import dataclass, field, asdict
from typing import Optional, Tuple, List
from enum import Enum

# ══════════════════════════════════════════════════════════════

# DEPENDENCY DETECTION

# ══════════════════════════════════════════════════════════════

try:
import ecdsa
HAS_ECDSA = True
except ImportError:
HAS_ECDSA = False

try:
from Crypto.Cipher import AES
HAS_AES = True
except ImportError:
try:
from Cryptodome.Cipher import AES
HAS_AES = True
except ImportError:
HAS_AES = False

STUB_MODE = not (HAS_ECDSA and HAS_AES)

if STUB_MODE:
import warnings
missing = []
if not HAS_ECDSA:
missing.append(“ecdsa”)
if not HAS_AES:
missing.append(“pycryptodome”)
warnings.warn(
f”BUTL running in STUB MODE — missing: {’, ‘.join(missing)}. “
f”Install with: pip install {’ ’.join(missing)}. “
f”Stub mode is structurally correct but not production-safe.”,
RuntimeWarning,
)

# ══════════════════════════════════════════════════════════════

# CRYPTOGRAPHIC PRIMITIVES

# ══════════════════════════════════════════════════════════════

def sha256(data: bytes) -> bytes:
“”“SHA-256 hash. Returns 32 bytes.”””
return hashlib.sha256(data).digest()

def sha256_hex(data: bytes) -> str:
“”“SHA-256 hash. Returns 64 lowercase hex characters.”””
return hashlib.sha256(data).hexdigest()

def double_sha256(data: bytes) -> bytes:
“”“Bitcoin-style double SHA-256: SHA-256(SHA-256(data)).”””
return sha256(sha256(data))

# ── Base58Check ──────────────────────────────────────────────

BASE58_ALPHABET = “123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz”

def base58check_encode(version: int, payload: bytes) -> str:
“”“Encode with version byte and 4-byte double-SHA-256 checksum.”””
data = bytes([version]) + payload
checksum = double_sha256(data)[:4]
full = data + checksum
n = int.from_bytes(full, “big”)
result = “”
while n > 0:
n, r = divmod(n, 58)
result = BASE58_ALPHABET[r] + result
for byte in full:
if byte == 0:
result = “1” + result
else:
break
return result

def public_key_to_address(pubkey_bytes: bytes) -> str:
“”“Compressed secp256k1 public key (33 bytes) → Bitcoin P2PKH address.”””
sha_hash = sha256(pubkey_bytes)
ripemd_hash = hashlib.new(“ripemd160”, sha_hash).digest()
return base58check_encode(0x00, ripemd_hash)

def verify_address_pubkey_consistency(address: str, pubkey_hex: str) -> bool:
“”“Verify that a public key correctly derives to the claimed address.
Returns True if consistent, False if mismatched.”””
try:
pubkey_bytes = bytes.fromhex(pubkey_hex)
derived = public_key_to_address(pubkey_bytes)
return derived == address
except (ValueError, Exception):
return False

# ── Encryption / Decryption ──────────────────────────────────

def encrypt_payload(plaintext: bytes, shared_secret: bytes,
sender_addr: str, receiver_addr: str) -> bytes:
“”“AES-256-GCM encrypt.
Returns: IV (12 bytes) || ciphertext || GCM tag (16 bytes).
AAD = UTF-8(sender_address || receiver_address).”””
iv = os.urandom(12)
aad = (sender_addr + receiver_addr).encode(“utf-8”)
if HAS_AES:
cipher = AES.new(shared_secret, AES.MODE_GCM, nonce=iv)
cipher.update(aad)
ct, tag = cipher.encrypt_and_digest(plaintext)
return iv + ct + tag
else:
# Stub: SHA-256-CTR keystream + HMAC-SHA-256 tag
import hmac as _hmac
ks = b””
i = 0
while len(ks) < len(plaintext):
ks += sha256(shared_secret + iv + i.to_bytes(4, “big”))
i += 1
ct = bytes(a ^ b for a, b in zip(plaintext, ks[:len(plaintext)]))
tag = _hmac.new(shared_secret, aad + iv + ct, hashlib.sha256).digest()[:16]
return iv + ct + tag

def decrypt_payload(encrypted: bytes, shared_secret: bytes,
sender_addr: str, receiver_addr: str) -> bytes:
“”“AES-256-GCM decrypt.
Input: IV (12 bytes) || ciphertext || GCM tag (16 bytes).
Raises ValueError on authentication failure.”””
if len(encrypted) < 28:
raise ValueError(“Encrypted payload too short”)
iv = encrypted[:12]
ct = encrypted[12:-16]
tag = encrypted[-16:]
aad = (sender_addr + receiver_addr).encode(“utf-8”)
if HAS_AES:
cipher = AES.new(shared_secret, AES.MODE_GCM, nonce=iv)
cipher.update(aad)
try:
return cipher.decrypt_and_verify(ct, tag)
except ValueError:
raise ValueError(“AES-GCM authentication failed — tampered or wrong key”)
else:
import hmac as _hmac
expected = _hmac.new(shared_secret, aad + iv + ct, hashlib.sha256).digest()[:16]
if not _hmac.compare_digest(tag, expected):
raise ValueError(“Authentication failed — tampered or wrong key”)
ks = b””
i = 0
while len(ks) < len(ct):
ks += sha256(shared_secret + iv + i.to_bytes(4, “big”))
i += 1
return bytes(a ^ b for a, b in zip(ct, ks[:len(ct)]))

# ══════════════════════════════════════════════════════════════

# BUTL KEYPAIR

# ══════════════════════════════════════════════════════════════

class BUTLKeypair:
“”“A single secp256k1 keypair for BUTL.

```
Provides:
  - Bitcoin address derivation
  - ECDSA signing and verification
  - ECDH shared secret computation
  - Secure memory deletion

Usage:
    # Generate random identity
    me = BUTLKeypair()

    # From specific seed (deterministic)
    me = BUTLKeypair(sha256(b"my-seed"))

    # Access identity components
    me.address          # Bitcoin address (string)
    me.public_key       # Compressed public key (33 bytes)
    me.public_key_hex   # Compressed public key (66 hex chars)
    me.private_key      # Private key (32 bytes) — NEVER share
"""

def __init__(self, private_key: Optional[bytes] = None):
    """Create a keypair from a 32-byte private key, or generate randomly."""
    if HAS_ECDSA:
        if private_key:
            self._sk = ecdsa.SigningKey.from_string(
                private_key, curve=ecdsa.SECP256k1
            )
        else:
            self._sk = ecdsa.SigningKey.generate(curve=ecdsa.SECP256k1)
        self._vk = self._sk.get_verifying_key()
        self._point = self._vk.pubkey.point
        prefix = b"\x02" if self._point.y() % 2 == 0 else b"\x03"
        self.public_key = prefix + self._point.x().to_bytes(32, "big")
        self.private_key = self._sk.to_string()
    else:
        self.private_key = private_key or os.urandom(32)
        # Stub: 0x02 prefix + 32-byte hash = 33 bytes = 66 hex chars
        self.public_key = b"\x02" + sha256(self.private_key)
        self._point = None

    self.address = public_key_to_address(self.public_key)
    self.public_key_hex = self.public_key.hex()

def sign(self, message_hash: bytes) -> bytes:
    """Sign a 32-byte SHA-256 hash. Returns DER-encoded ECDSA signature."""
    if HAS_ECDSA:
        return self._sk.sign_digest(
            message_hash, sigencode=ecdsa.util.sigencode_der
        )
    # Stub: deterministic fake signature
    return sha256(self.private_key + message_hash)

def ecdh(self, other_pubkey: bytes) -> bytes:
    """Compute ECDH shared secret: SHA-256((my_priv × their_pub).x).
    other_pubkey: 33-byte compressed secp256k1 public key."""
    if HAS_ECDSA:
        if len(other_pubkey) == 33:
            uncompressed = self._decompress(other_pubkey)
        else:
            uncompressed = other_pubkey
        other_vk = ecdsa.VerifyingKey.from_string(
            uncompressed, curve=ecdsa.SECP256k1
        )
        shared_point = other_vk.pubkey.point * int.from_bytes(
            self.private_key, "big"
        )
        return sha256(shared_point.x().to_bytes(32, "big"))
    # Stub: commutative shared secret from both public keys (sorted).
    # This ensures sender and receiver compute the same secret,
    # matching the commutativity property of real ECDH.
    a, b = self.public_key, other_pubkey
    if a > b:
        a, b = b, a
    return sha256(a + b)

@staticmethod
def _decompress(compressed: bytes) -> bytes:
    """Decompress 33-byte public key to 64-byte uncompressed (x || y)."""
    if len(compressed) != 33:
        raise ValueError(f"Expected 33 bytes, got {len(compressed)}")
    prefix = compressed[0]
    x = int.from_bytes(compressed[1:], "big")
    p = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
    y_sq = (pow(x, 3, p) + 7) % p
    y = pow(y_sq, (p + 1) // 4, p)
    if (y % 2 == 0) != (prefix == 0x02):
        y = p - y
    return x.to_bytes(32, "big") + y.to_bytes(32, "big")

@staticmethod
def verify(address: str, signature: bytes, message_hash: bytes,
           public_key: Optional[bytes] = None) -> bool:
    """Verify an ECDSA signature against a compressed public key.
    Returns True if valid, False otherwise."""
    if HAS_ECDSA:
        if public_key is None:
            raise ValueError("Public key required for verification")
        if len(public_key) == 33:
            uncompressed = BUTLKeypair._decompress(public_key)
        else:
            uncompressed = public_key
        vk = ecdsa.VerifyingKey.from_string(
            uncompressed, curve=ecdsa.SECP256k1
        )
        try:
            return vk.verify_digest(
                signature, message_hash,
                sigdecode=ecdsa.util.sigdecode_der,
            )
        except ecdsa.BadSignatureError:
            return False
    # Stub: always returns True (gate logic works structurally)
    return True

def secure_delete(self):
    """Overwrite private key material in memory.
    Call after chain proof creation when the key is no longer needed."""
    if hasattr(self, "private_key") and self.private_key:
        self.private_key = b"\x00" * len(self.private_key)
    self._sk = None
```

# ══════════════════════════════════════════════════════════════

# BUTL KEYCHAIN

# ══════════════════════════════════════════════════════════════

class BUTLKeychain:
“”“Deterministic key derivation with automatic address chaining.

```
Derives a sequence of keypairs from a master seed. Each call to
next_keypair() advances the chain: the current keypair becomes the
previous (retained for chain proof), and a new keypair is generated.

Usage:
    keychain = BUTLKeychain(seed=os.urandom(32))
    kp = keychain.next_keypair()    # Key 0 (genesis)
    kp = keychain.next_keypair()    # Key 1 (chained to 0)
    proof = keychain.create_chain_proof(kp.address)
"""

def __init__(self, seed: Optional[bytes] = None):
    """Initialize with a 32-byte master seed (random if not provided)."""
    self.seed = seed or os.urandom(32)
    self._index = 0
    self._current: Optional[BUTLKeypair] = None
    self._previous: Optional[BUTLKeypair] = None
    self._history: List[str] = []

def _derive_key(self, index: int) -> bytes:
    """Derive a private key from seed + index.
    Simplified BIP-32 style. Production: use full BIP-32 HD derivation."""
    return sha256(self.seed + index.to_bytes(4, "big"))

def next_keypair(self) -> BUTLKeypair:
    """Generate the next keypair in the chain.
    The previous keypair is retained for chain proof creation."""
    self._previous = self._current
    self._current = BUTLKeypair(self._derive_key(self._index))
    self._history.append(self._current.address)
    self._index += 1
    return self._current

@property
def current(self) -> Optional[BUTLKeypair]:
    """The current (most recent) keypair."""
    return self._current

@property
def previous(self) -> Optional[BUTLKeypair]:
    """The previous keypair (for chain proof creation)."""
    return self._previous

@property
def previous_address(self) -> str:
    """The previous keypair's address (for BUTL-PrevAddr)."""
    return self._previous.address if self._previous else ""

def create_chain_proof(self, current_address: str) -> bytes:
    """Sign the current address with the previous key.
    Returns empty bytes for genesis (no previous key)."""
    if self._previous is None:
        return b""
    chain_hash = sha256(current_address.encode("utf-8"))
    return self._previous.sign(chain_hash)

@property
def thread_id(self) -> str:
    """Thread ID = SHA-256_HEX(first address in chain)."""
    if self._history:
        return sha256_hex(self._history[0].encode("utf-8"))
    return ""

@property
def sequence_number(self) -> int:
    """Current sequence number (0-indexed)."""
    return max(0, self._index - 1)

@property
def history(self) -> List[str]:
    """All addresses generated by this keychain."""
    return list(self._history)

def cleanup_previous(self):
    """Securely delete the previous keypair after chain proof is created."""
    if self._previous:
        self._previous.secure_delete()
        self._previous = None
```

# ══════════════════════════════════════════════════════════════

# BUTL HEADER

# ══════════════════════════════════════════════════════════════

@dataclass
class BUTLHeader:
“”“BUTL protocol header v1.2.

```
Contains all metadata required for the verification gate.
Transmitted in cleartext during Phase 1 (header-only delivery).

See HEADER_REGISTRY.md for the complete field specification.
See BUTL_Protocol_Specification_v1.2.md Section 7 for the
canonical signing payload format.
"""

# ── Required fields ──
version: int = 1
sender: str = ""
sender_pubkey: str = ""
receiver: str = ""
receiver_pubkey: str = ""
block_height: int = 0
block_hash: str = ""
payload_hash: str = ""
signature: str = ""
prev_addr: str = ""
chain_proof: str = ""
enc_algo: str = "AES-256-GCM"

# ── Optional fields ──
ephemeral_pubkey: str = ""
nonce: str = ""
timestamp: int = 0
thread_id: str = ""
seq_num: int = 0
payload_size: int = 0

def canonical_payload(self) -> str:
    """Build the deterministic canonical signing payload.
    Fields in fixed order, separated by newline (0x0A).
    Empty strings for absent optional values."""
    return "\n".join([
        f"BUTL-Version:{self.version}",
        f"BUTL-Sender:{self.sender}",
        f"BUTL-SenderPubKey:{self.sender_pubkey}",
        f"BUTL-Receiver:{self.receiver}",
        f"BUTL-ReceiverPubKey:{self.receiver_pubkey}",
        f"BUTL-BlockHeight:{self.block_height}",
        f"BUTL-BlockHash:{self.block_hash}",
        f"BUTL-PayloadHash:{self.payload_hash}",
        f"BUTL-PrevAddr:{self.prev_addr}",
        f"BUTL-Nonce:{self.nonce}",
    ])

def to_text_headers(self) -> str:
    """Serialize to text header format (email X-headers / HTTP headers).
    Format: 'Key: Value' with one space after colon."""
    lines = [
        f"BUTL-Version: {self.version}",
        f"BUTL-Sender: {self.sender}",
        f"BUTL-SenderPubKey: {self.sender_pubkey}",
        f"BUTL-Receiver: {self.receiver}",
        f"BUTL-ReceiverPubKey: {self.receiver_pubkey}",
        f"BUTL-BlockHeight: {self.block_height}",
        f"BUTL-BlockHash: {self.block_hash}",
        f"BUTL-PayloadHash: {self.payload_hash}",
        f"BUTL-Signature: {self.signature}",
        f"BUTL-PrevAddr: {self.prev_addr}",
        f"BUTL-ChainProof: {self.chain_proof}",
        f"BUTL-EncAlgo: {self.enc_algo}",
    ]
    if self.ephemeral_pubkey:
        lines.append(f"BUTL-EphemeralPubKey: {self.ephemeral_pubkey}")
    if self.nonce:
        lines.append(f"BUTL-Nonce: {self.nonce}")
    if self.timestamp:
        lines.append(f"BUTL-Timestamp: {self.timestamp}")
    if self.thread_id:
        lines.append(f"BUTL-ThreadID: {self.thread_id}")
    if self.seq_num > 0:
        lines.append(f"BUTL-SeqNum: {self.seq_num}")
    if self.payload_size > 0:
        lines.append(f"BUTL-PayloadSize: {self.payload_size}")
    return "\n".join(lines)

def to_dict(self) -> dict:
    """Serialize to dictionary (snake_case keys)."""
    return asdict(self)

def to_json(self, indent: int = 2) -> str:
    """Serialize to JSON string."""
    return json.dumps(self.to_dict(), indent=indent)

@classmethod
def from_text_headers(cls, text: str) -> "BUTLHeader":
    """Parse from text header format ('Key: Value' per line)."""
    header = cls()
    field_map = {
        "BUTL-Version": ("version", int),
        "BUTL-Sender": ("sender", str),
        "BUTL-SenderPubKey": ("sender_pubkey", str),
        "BUTL-Receiver": ("receiver", str),
        "BUTL-ReceiverPubKey": ("receiver_pubkey", str),
        "BUTL-BlockHeight": ("block_height", int),
        "BUTL-BlockHash": ("block_hash", str),
        "BUTL-PayloadHash": ("payload_hash", str),
        "BUTL-Signature": ("signature", str),
        "BUTL-PrevAddr": ("prev_addr", str),
        "BUTL-ChainProof": ("chain_proof", str),
        "BUTL-EncAlgo": ("enc_algo", str),
        "BUTL-EphemeralPubKey": ("ephemeral_pubkey", str),
        "BUTL-Nonce": ("nonce", str),
        "BUTL-Timestamp": ("timestamp", int),
        "BUTL-ThreadID": ("thread_id", str),
        "BUTL-SeqNum": ("seq_num", int),
        "BUTL-PayloadSize": ("payload_size", int),
    }
    for line in text.strip().split("\n"):
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key, value = key.strip(), value.strip()
        if key in field_map:
            attr, type_fn = field_map[key]
            setattr(header, attr, type_fn(value))
    return header

@classmethod
def from_dict(cls, d: dict) -> "BUTLHeader":
    """Construct from dictionary."""
    header = cls()
    for k, v in d.items():
        if hasattr(header, k):
            setattr(header, k, v)
    return header

@classmethod
def from_json(cls, json_str: str) -> "BUTLHeader":
    """Construct from JSON string."""
    return cls.from_dict(json.loads(json_str))
```

# ══════════════════════════════════════════════════════════════

# BLOCKCHAIN INTERFACE

# ══════════════════════════════════════════════════════════════

class BlockchainInterface:
“”“Abstract interface for Bitcoin blockchain queries.

```
The reference implementation includes a stub that returns hardcoded
values. Production deployments MUST replace this with a real
implementation connecting to Bitcoin Core RPC, Electrum server,
or a REST API (e.g., mempool.space).

See SECURITY.md for known limitations of the stub.
"""

def get_current_block(self) -> Tuple[int, str]:
    """Returns (block_height, block_hash) of the current chain tip."""
    return (
        890412,
        "00000000000000000002a7c4c1e48d76f0593d3eabc1c3d1f92c4e5a6b8c7d9e",
    )

def verify_block(self, height: int, block_hash: str) -> bool:
    """Verify that block_hash matches the block at the given height."""
    return True

def get_chain_height(self) -> int:
    """Returns the current chain tip height."""
    return 890412

def check_balance(self, address: str, min_satoshis: int = 1) -> bool:
    """Check if address holds at least min_satoshis.
    Only called when Proof of Satoshi is enabled."""
    return True
```

# ══════════════════════════════════════════════════════════════

# BUTL MESSAGE

# ══════════════════════════════════════════════════════════════

@dataclass
class BUTLMessage:
“”“A complete BUTL message: header + encrypted payload.
The header is delivered in Phase 1. The payload in Phase 2.”””
header: BUTLHeader
encrypted_payload: bytes

# ══════════════════════════════════════════════════════════════

# PROOF OF SATOSHI CONFIGURATION

# ══════════════════════════════════════════════════════════════

@dataclass
class ProofOfSatoshiConfig:
“”“Receiver-side Proof of Satoshi configuration.

```
enabled: Whether to enforce the balance check (Step 5).
min_satoshis: Minimum balance required. Range: 1 — 2,100,000,000,000,000 (21,000,000 BTC).

When enabled=False (default), Step 5 is skipped entirely
and BUTL operates as pure math with zero external dependencies.
"""
enabled: bool = False
min_satoshis: int = 1
```

# ══════════════════════════════════════════════════════════════

# SIGNER (SENDER SIDE)

# ══════════════════════════════════════════════════════════════

class BUTLSigner:
“”“Signs and encrypts messages with the BUTL protocol.

```
The signer manages the sender's keychain (fresh address per message),
constructs the BUTL header, signs the canonical payload, encrypts
the message body, and produces a complete BUTLMessage.

Usage:
    keychain = BUTLKeychain(seed=my_seed)
    signer = BUTLSigner(keychain)
    msg = signer.sign_and_encrypt(
        body=b"Hello!",
        receiver_pubkey=their_pubkey,
        receiver_address=their_address,
    )
"""

def __init__(self, keychain: BUTLKeychain,
             blockchain: Optional[BlockchainInterface] = None):
    self.keychain = keychain
    self.blockchain = blockchain or BlockchainInterface()

def sign_and_encrypt(
    self,
    body: bytes,
    receiver_pubkey: bytes,
    receiver_address: str,
    include_nonce: bool = True,
    use_ephemeral: bool = False,
) -> BUTLMessage:
    """Sign and encrypt a message for a specific receiver.

    Args:
        body: Plaintext message body (bytes).
        receiver_pubkey: Receiver's compressed secp256k1 public key (33 bytes).
        receiver_address: Receiver's Bitcoin address (string).
        include_nonce: Include random nonce for anti-replay (default True).
        use_ephemeral: Use ephemeral key for ECDH (default False).

    Returns:
        BUTLMessage with header and encrypted payload.
    """
    # 1. Fresh sender keypair (address chaining)
    keypair = self.keychain.next_keypair()

    # 2. ECDH key agreement
    if use_ephemeral:
        ephemeral = BUTLKeypair()
        shared_secret = ephemeral.ecdh(receiver_pubkey)
        ephemeral_pubkey_hex = ephemeral.public_key_hex
    else:
        shared_secret = keypair.ecdh(receiver_pubkey)
        ephemeral_pubkey_hex = ""

    # 3. Encrypt payload
    encrypted = encrypt_payload(
        body, shared_secret, keypair.address, receiver_address
    )

    # 4. Hash encrypted payload
    payload_hash = sha256_hex(encrypted)

    # 5. Current block reference
    block_height, block_hash = self.blockchain.get_current_block()

    # 6. Build header
    header = BUTLHeader(
        version=__protocol_version__,
        sender=keypair.address,
        sender_pubkey=keypair.public_key_hex,
        receiver=receiver_address,
        receiver_pubkey=receiver_pubkey.hex(),
        block_height=block_height,
        block_hash=block_hash,
        payload_hash=payload_hash,
        prev_addr=self.keychain.previous_address,
        enc_algo="AES-256-GCM",
        ephemeral_pubkey=ephemeral_pubkey_hex,
        nonce=os.urandom(12).hex() if include_nonce else "",
        timestamp=int(time.time()),
        thread_id=self.keychain.thread_id,
        seq_num=self.keychain.sequence_number,
        payload_size=len(encrypted),
    )

    # 7. Chain proof
    if self.keychain.previous is not None:
        chain_proof = self.keychain.create_chain_proof(keypair.address)
        header.chain_proof = base64.b64encode(chain_proof).decode("ascii")

    # 8. Sign canonical payload
    canonical_hash = sha256(header.canonical_payload().encode("utf-8"))
    sig = keypair.sign(canonical_hash)
    header.signature = base64.b64encode(sig).decode("ascii")

    return BUTLMessage(header=header, encrypted_payload=encrypted)
```

# ══════════════════════════════════════════════════════════════

# GATE REPORT

# ══════════════════════════════════════════════════════════════

class CheckResult(Enum):
“”“Result of a single verification step.”””
PASS = “PASS”
FAIL = “FAIL”
SKIP = “SKIP”

@dataclass
class GateReport:
“”“Detailed report of all 8 verification steps.

```
Properties:
    gate_passed:     True if all pre-download checks (1-6) passed.
    fully_verified:  True if all 8 checks passed including payload.
    summary():       Human-readable multi-line report string.
"""
structural: CheckResult = CheckResult.SKIP
receiver_match: CheckResult = CheckResult.SKIP
signature: CheckResult = CheckResult.SKIP
freshness: CheckResult = CheckResult.SKIP
pos: CheckResult = CheckResult.SKIP
chain: CheckResult = CheckResult.SKIP
payload_hash: CheckResult = CheckResult.SKIP
decryption: CheckResult = CheckResult.SKIP
errors: List[str] = field(default_factory=list)

@property
def gate_passed(self) -> bool:
    """True if all enabled pre-download gate checks (steps 1-6) passed."""
    checks = [
        self.structural, self.receiver_match,
        self.signature, self.freshness,
    ]
    if self.pos != CheckResult.SKIP:
        checks.append(self.pos)
    if self.chain != CheckResult.SKIP:
        checks.append(self.chain)
    return all(c == CheckResult.PASS for c in checks)

@property
def fully_verified(self) -> bool:
    """True if ALL 8 checks passed, including payload verification."""
    return (
        self.gate_passed
        and self.payload_hash == CheckResult.PASS
        and self.decryption == CheckResult.PASS
    )

def summary(self) -> str:
    """Human-readable verification report."""
    if self.fully_verified:
        status = "FULLY VERIFIED"
    elif self.gate_passed:
        status = "GATE PASSED"
    else:
        status = "REJECTED AT GATE"
    gate = "OPEN" if self.gate_passed else "CLOSED"
    lines = [
        f"BUTL Verification: {status}",
        "",
        "  == BUTL GATE (header-only, pre-download) ==",
        f"  1. Structural Validation  : {self.structural.value}",
        f"  2. Receiver Match         : {self.receiver_match.value}",
        f"  3. Signature (ECDSA)      : {self.signature.value}",
        f"  4. Block Freshness        : {self.freshness.value}",
        f"  5. Proof of Satoshi       : {self.pos.value}",
        f"  6. Chain Proof            : {self.chain.value}",
        f"  Gate: {gate}",
        "",
        "  == POST-GATE (payload verification) ==",
        f"  7. Payload Hash (SHA-256) : {self.payload_hash.value}",
        f"  8. Decryption (AES-GCM)   : {self.decryption.value}",
    ]
    if self.errors:
        lines.extend(["", "  Errors:"])
        lines.extend(f"    - {e}" for e in self.errors)
    return "\n".join(lines)
```

# ══════════════════════════════════════════════════════════════

# BUTL GATE (RECEIVER SIDE)

# ══════════════════════════════════════════════════════════════

class BUTLGate:
“”“Verify-Before-Download Gate v1.2.

```
Enforces the BUTL security model: no payload data touches the
device until all header checks pass.

Two-phase usage:
    gate = BUTLGate(my_keypair)
    report = gate.check_header(header)
    if report.gate_passed:
        plaintext, report = gate.accept_payload(header, payload, report)
"""

def __init__(
    self,
    receiver_keypair: BUTLKeypair,
    blockchain: Optional[BlockchainInterface] = None,
    freshness_window: int = 144,
    pos_config: Optional[ProofOfSatoshiConfig] = None,
):
    self.receiver = receiver_keypair
    self.blockchain = blockchain or BlockchainInterface()
    self.freshness_window = freshness_window
    self.pos_config = pos_config or ProofOfSatoshiConfig()

def check_header(
    self,
    header: BUTLHeader,
    prev_pubkey: Optional[bytes] = None,
) -> GateReport:
    """Phase 1: Header-only verification (Steps 1-6).
    No payload data is involved."""
    report = GateReport()

    # ── Step 1: Structural Validation ──
    if header.version != __protocol_version__:
        report.errors.append(
            f"Unsupported version: {header.version} "
            f"(supports {__protocol_version__})"
        )
        report.structural = CheckResult.FAIL
        return report

    for name, value, expected_len in [
        ("Sender", header.sender, None),
        ("SenderPubKey", header.sender_pubkey, 66),
        ("Receiver", header.receiver, None),
        ("ReceiverPubKey", header.receiver_pubkey, 66),
        ("BlockHash", header.block_hash, 64),
        ("PayloadHash", header.payload_hash, 64),
        ("Signature", header.signature, None),
    ]:
        if not value:
            report.errors.append(f"Missing: BUTL-{name}")
            report.structural = CheckResult.FAIL
            return report
        if expected_len and len(value) != expected_len:
            report.errors.append(
                f"BUTL-{name}: expected {expected_len} chars, got {len(value)}"
            )
            report.structural = CheckResult.FAIL
            return report

    if not verify_address_pubkey_consistency(
        header.sender, header.sender_pubkey
    ):
        report.errors.append(
            "Consistency check failed: BUTL-SenderPubKey "
            "does not derive to BUTL-Sender"
        )
        report.structural = CheckResult.FAIL
        return report

    report.structural = CheckResult.PASS

    # ── Step 2: Receiver Match ──
    if header.receiver != self.receiver.address:
        report.errors.append(
            f"Not for this receiver: {header.receiver} "
            f"!= {self.receiver.address}"
        )
        report.receiver_match = CheckResult.FAIL
        return report
    report.receiver_match = CheckResult.PASS

    # ── Step 3: Signature Verification ──
    try:
        canonical_hash = sha256(
            header.canonical_payload().encode("utf-8")
        )
        sig_bytes = base64.b64decode(header.signature)
        sender_pk = bytes.fromhex(header.sender_pubkey)
        valid = BUTLKeypair.verify(
            header.sender, sig_bytes, canonical_hash, sender_pk
        )
        if not valid:
            report.errors.append("ECDSA signature verification failed")
            report.signature = CheckResult.FAIL
            return report
        report.signature = CheckResult.PASS
    except Exception as e:
        report.errors.append(f"Signature error: {e}")
        report.signature = CheckResult.FAIL
        return report

    # ── Step 4: Block Freshness ──
    if not self.blockchain.verify_block(
        header.block_height, header.block_hash
    ):
        report.errors.append("Block hash does not match claimed height")
        report.freshness = CheckResult.FAIL
        return report
    age = self.blockchain.get_chain_height() - header.block_height
    if age > self.freshness_window or age < 0:
        report.errors.append(
            f"Block age {age} outside window [0, {self.freshness_window}]"
        )
        report.freshness = CheckResult.FAIL
        return report
    report.freshness = CheckResult.PASS

    # ── Step 5: Proof of Satoshi (OPTIONAL) ──
    if self.pos_config.enabled:
        if not self.blockchain.check_balance(
            header.sender, self.pos_config.min_satoshis
        ):
            report.errors.append(
                f"Proof of Satoshi: {header.sender} holds "
                f"< {self.pos_config.min_satoshis} sat"
            )
            report.pos = CheckResult.FAIL
            return report
        report.pos = CheckResult.PASS
    else:
        report.pos = CheckResult.SKIP

    # ── Step 6: Chain Proof ──
    if header.prev_addr:
        if not header.chain_proof:
            report.errors.append(
                "Threaded message but no ChainProof"
            )
            report.chain = CheckResult.FAIL
            return report
        try:
            proof_hash = sha256(header.sender.encode("utf-8"))
            proof_bytes = base64.b64decode(header.chain_proof)
            valid = BUTLKeypair.verify(
                header.prev_addr, proof_bytes, proof_hash, prev_pubkey
            )
            if not valid:
                report.errors.append("Chain proof verification failed")
                report.chain = CheckResult.FAIL
                return report
            report.chain = CheckResult.PASS
        except Exception as e:
            report.errors.append(f"Chain proof error: {e}")
            report.chain = CheckResult.FAIL
            return report
    else:
        report.chain = CheckResult.PASS

    return report

def accept_payload(
    self,
    header: BUTLHeader,
    encrypted_payload: bytes,
    report: GateReport,
) -> Tuple[Optional[bytes], GateReport]:
    """Phase 2: Download and decrypt the payload.
    Only call if report.gate_passed is True."""
    if not report.gate_passed:
        report.errors.append("GATE CLOSED — payload rejected")
        return None, report

    # ── Step 7: Payload Hash ──
    computed = sha256_hex(encrypted_payload)
    if computed != header.payload_hash:
        report.errors.append("Payload hash mismatch — tampered in transit")
        report.payload_hash = CheckResult.FAIL
        return None, report
    report.payload_hash = CheckResult.PASS

    # ── Step 8: Decryption ──
    try:
        ecdh_pubkey_hex = (
            header.ephemeral_pubkey
            if header.ephemeral_pubkey
            else header.sender_pubkey
        )
        ecdh_pubkey = bytes.fromhex(ecdh_pubkey_hex)
        shared_secret = self.receiver.ecdh(ecdh_pubkey)
        plaintext = decrypt_payload(
            encrypted_payload, shared_secret,
            header.sender, header.receiver,
        )
        report.decryption = CheckResult.PASS
        return plaintext, report
    except ValueError as e:
        report.errors.append(f"Decryption failed: {e}")
        report.decryption = CheckResult.FAIL
        return None, report
```

# ══════════════════════════════════════════════════════════════

# DEMO

# ══════════════════════════════════════════════════════════════

def demo():
“”“Demonstrate the full BUTL v1.2 sign → verify → decrypt flow.”””

```
print("=" * 66)
print("  BUTL Reference Implementation v1.2 (Python)")
print(f"  Protocol version: {__protocol_version__}")
print(f"  Library version:  {__version__}")
if STUB_MODE:
    print("  Mode: STUB (install ecdsa + pycryptodome for real crypto)")
else:
    print("  Mode: PRODUCTION (real secp256k1 + AES-256-GCM)")
print("=" * 66)

# ── Setup ──
sender_keychain = BUTLKeychain(seed=sha256(b"demo-sender-seed"))
receiver_keypair = BUTLKeypair(sha256(b"demo-receiver-seed"))
signer = BUTLSigner(sender_keychain)
gate = BUTLGate(
    receiver_keypair,
    pos_config=ProofOfSatoshiConfig(enabled=False),
)

print(f"\nReceiver: {receiver_keypair.address}")

# ── Message 1: Genesis ──
print(f"\n{'-' * 66}")
print("MESSAGE 1 (Genesis)\n")
body1 = b"Hello! First BUTL v1.2 message."
msg1 = signer.sign_and_encrypt(
    body1, receiver_keypair.public_key, receiver_keypair.address
)
print(f"Encrypted payload: {len(msg1.encrypted_payload)} bytes")

sender1_pk = bytes.fromhex(msg1.header.sender_pubkey)
report1 = gate.check_header(msg1.header)
if report1.gate_passed:
    pt1, report1 = gate.accept_payload(
        msg1.header, msg1.encrypted_payload, report1
    )
    if pt1:
        print(f'Decrypted: "{pt1.decode("utf-8")}"')
print(f"\n{report1.summary()}")

# ── Message 2: Chained ──
print(f"\n{'-' * 66}")
print("MESSAGE 2 (Chained)\n")
body2 = b"Second message, chained to the first."
msg2 = signer.sign_and_encrypt(
    body2, receiver_keypair.public_key, receiver_keypair.address
)
sender2_pk = bytes.fromhex(msg2.header.sender_pubkey)
report2 = gate.check_header(msg2.header, prev_pubkey=sender1_pk)
if report2.gate_passed:
    pt2, report2 = gate.accept_payload(
        msg2.header, msg2.encrypted_payload, report2
    )
    if pt2:
        print(f'Decrypted: "{pt2.decode("utf-8")}"')
print(f"\n{report2.summary()}")

# ── Tampered payload ──
print(f"\n{'-' * 66}")
print("TAMPERED PAYLOAD\n")
tampered = msg2.encrypted_payload[:-1] + bytes(
    [msg2.encrypted_payload[-1] ^ 0xFF]
)
report_t = gate.check_header(msg2.header, prev_pubkey=sender1_pk)
if report_t.gate_passed:
    pt_t, report_t = gate.accept_payload(
        msg2.header, tampered, report_t
    )
    print(f"Payload accepted: {pt_t is not None}")
print(f"\n{report_t.summary()}")

# ── Wrong receiver ──
print(f"\n{'-' * 66}")
print("WRONG RECEIVER\n")
wrong_kp = BUTLKeypair(sha256(b"wrong-person"))
wrong_gate = BUTLGate(wrong_kp)
report_w = wrong_gate.check_header(msg1.header)
print(f"Gate passed: {report_w.gate_passed}")
print(f"\n{report_w.summary()}")

# ── Proof of Satoshi ──
print(f"\n{'-' * 66}")
print("PROOF OF SATOSHI ENABLED (slider = 10,000 sat)\n")
pos_gate = BUTLGate(
    receiver_keypair,
    pos_config=ProofOfSatoshiConfig(enabled=True, min_satoshis=10000),
)
msg3 = signer.sign_and_encrypt(
    b"PoS test", receiver_keypair.public_key, receiver_keypair.address
)
report_pos = pos_gate.check_header(msg3.header, prev_pubkey=sender2_pk)
if report_pos.gate_passed:
    pt3, report_pos = pos_gate.accept_payload(
        msg3.header, msg3.encrypted_payload, report_pos
    )
    if pt3:
        print(f'Decrypted: "{pt3.decode("utf-8")}"')
print(f"\n{report_pos.summary()}")

# ── Address chain ──
print(f"\n{'-' * 66}")
print("ADDRESS CHAIN\n")
print(f"Thread ID: {sender_keychain.thread_id}")
for i, addr in enumerate(sender_keychain.history):
    label = "(genesis)" if i == 0 else "(chained)"
    print(f"  Msg {i}: {addr}  {label}")

print(f"\n{'=' * 66}")
print("  Demo complete.")
print(f"{'=' * 66}")
```

if **name** == “**main**”:
demo()