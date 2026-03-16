# How to Prove the BUTL Protocol Works

## A Step-by-Step Guide for Complete Beginners

**No coding experience required.** This guide assumes you have never written a line of code. You will download one file, type one command, and watch the BUTL protocol prove itself — on your machine, with your own eyes.

-----

## What You Will Prove

By following this guide, you will personally verify that the BUTL Protocol delivers all 8 of its promised capabilities:

1. **Message integrity** — tampering with a message is detected instantly
1. **Bitcoin identity** — the sender has a real Bitcoin address derived from real cryptography
1. **Digital signatures** — only the sender’s private key can produce a valid signature
1. **Replay protection** — old messages are rejected (block height freshness)
1. **Sybil resistance** — the optional Proof of Satoshi balance check works (4 scenarios)
1. **Forward privacy** — a new address is used for every message, cryptographically linked to the last
1. **End-to-end encryption** — only the intended receiver can decrypt the message
1. **Verify-before-download** — no payload touches the device until all checks pass

You will also see:

- A **consistency check** proving that the public key correctly derives to the Bitcoin address
- A **tampered payload** being caught and rejected
- A **wrong receiver** being blocked at the gate — payload never downloaded

-----

## What You Need

- A computer (Mac, Windows, or Linux)
- Python 3.8 or newer (almost every computer made in the last 10 years has this)
- The file `butl_mvp_v12.py` from this repository
- No internet connection (the proof runs entirely offline)
- No accounts, no passwords, no signups
- No additional software to install

-----

## Step 1: Check If You Have Python

### Mac

Open **Terminal** (search “Terminal” in Spotlight, or find it in Applications → Utilities).

Type this and press Enter:

```bash
python3 --version
```

You should see something like `Python 3.10.4` or `Python 3.12.1`. Any version 3.8 or higher works.

If you see “command not found,” install Python from [python.org/downloads](https://www.python.org/downloads/).

### Windows

Open **Command Prompt** (search “cmd” in the Start menu).

Type this and press Enter:

```cmd
python --version
```

or:

```cmd
python3 --version
```

You should see `Python 3.x.x`. If you see “not recognized,” install Python from [python.org/downloads](https://www.python.org/downloads/). During installation, check the box that says “Add Python to PATH.”

### Linux

Open a terminal. Type:

```bash
python3 --version
```

Python is pre-installed on most Linux distributions. If not: `sudo apt install python3` (Ubuntu/Debian) or `sudo dnf install python3` (Fedora).

-----

## Step 2: Get the File

Download `butl_mvp_v12.py` from this repository. It’s in the `mvp/` folder.

You can also find it at: `https://github.com/[username]/butl-protocol/blob/main/mvp/butl_mvp_v12.py`

Save it somewhere easy to find. Your Desktop is fine.

**That’s the only file you need.** It has zero external dependencies — everything is built from scratch inside the file, including the entire secp256k1 elliptic curve math that Bitcoin uses.

-----

## Step 3: Run the Proof

### Mac / Linux

Open Terminal. Navigate to where you saved the file:

```bash
cd ~/Desktop
```

Run it:

```bash
python3 butl_mvp_v12.py
```

### Windows

Open Command Prompt. Navigate to where you saved the file:

```cmd
cd %USERPROFILE%\Desktop
```

Run it:

```cmd
python butl_mvp_v12.py
```

or:

```cmd
python3 butl_mvp_v12.py
```

-----

## Step 4: Read the Output

The program runs 8 proofs in sequence. Each proof demonstrates one capability of the BUTL protocol. Here’s what to look for:

### PROOF 1: SHA-256 Body Integrity

```
  Original hash:  755feaf841698...
  Tampered hash:  c69bf924a556b...
  Match: NO
  PROVED: Any modification to the body changes the hash.
```

**What this means:** The message body is hashed with SHA-256. Changing even one character produces a completely different hash. If anyone tampers with the message in transit, the hash won’t match and the tampering is detected.

### PROOF 2: Bitcoin Address Identity

```
  Address:    1L3XLqfhm9tojQLRPFGqAaYbif3ow4NcuR
  Public Key: 03a78fddfd26c7...
  Consistency check: PASS (pubkey derives to address)
  PROVED: Valid Bitcoin P2PKH address from secp256k1.
```

**What this means:** A real Bitcoin address was generated from a real secp256k1 keypair. The address starts with “1” (valid P2PKH format). The consistency check confirms the public key mathematically derives to that exact address — they can’t be faked independently.

### PROOF 3: ECDSA Signature Verification

```
  Verify with correct key: VALID
  Verify with wrong key:   INVALID
  PROVED: Only the private key holder can sign.
```

**What this means:** The sender signed a message with their private key. The receiver verified it with the sender’s public key — valid. An attacker tried to verify with a different key — invalid. Only the person with the private key can produce the signature.

### PROOF 4: Block Height Freshness

```
  Age 0 blocks:    ACCEPTED (current block)
  Age 100 blocks:  ACCEPTED (within window)
  Age 1412 blocks: REJECTED (stale — possible replay)
  PROVED: Messages outside the freshness window are rejected.
```

**What this means:** Each BUTL message references a recent Bitcoin block height. The receiver checks that the block isn’t too old. Messages within 144 blocks (~24 hours) are accepted. Older messages are rejected — this prevents replay attacks where an attacker re-sends an old message.

### PROOF 5: Proof of Satoshi (4 Scenarios)

```
  Scenario A: PoS DISABLED → Gate: OPEN
  Scenario B: PoS ENABLED, 1,000 sat threshold, sender has 50,000 → Gate: OPEN
  Scenario C: PoS ENABLED, 100,000 sat threshold, sender has 50,000 → Gate: CLOSED
  Scenario D: PoS ENABLED, zero balance → Gate: CLOSED
```

**What this means:** The balance check is optional. The receiver controls whether it’s on or off and what the minimum is.

- **Scenario A:** Disabled. The gate opens on pure math alone. No blockchain query needed.
- **Scenario B:** Enabled at 1,000 satoshis. The sender has 50,000. Passes.
- **Scenario C:** Enabled at 100,000 satoshis. The sender only has 50,000. Rejected.
- **Scenario D:** Enabled at 1 satoshi. The sender has zero. Rejected.

### PROOF 6: Address Chaining

```
  Message 0: 1L3XLqfhm9toj...
  Message 1: 1HshpDPFhoXrG...
  Chain proof (key_0 signs addr_1): VALID
  Forged proof (wrong key): INVALID
  PROVED: Fresh address per message, cryptographically linked.
```

**What this means:** Each message uses a different Bitcoin address. But the sender proves it’s still them by signing the new address with the old key. An attacker who tries to forge this proof fails because they don’t have the old private key.

### PROOF 7: Receiver-Only Encryption

```
  Shared secrets match: YES
  Receiver decrypts: "Hello from BUTL v1.2!"
  Wrong receiver: AUTH FAILED
  PROVED: Only the intended receiver can decrypt.
```

**What this means:** The sender and receiver each compute the same shared secret using ECDH (Elliptic Curve Diffie-Hellman). The message is encrypted with this secret. The intended receiver decrypts it successfully. A different person (wrong private key) computes a different secret and gets AUTH FAILED — they cannot read the message.

### PROOF 8: Verify-Before-Download Gate

```
  PHASE 1: Header Only
  1. Structural Validation: PASS
  2. Receiver Match: PASS
  3. Signature (ECDSA): PASS
  4. Block Freshness: PASS
  5. Proof of Satoshi [DISABLED]
  6. Chain Proof: PASS
  Gate: OPEN

  PHASE 2: Payload
  7. Payload Hash (SHA-256): PASS
  8. Decryption (AEAD): PASS
     Plaintext: "Hello from BUTL v1.2!"

  Tampered payload: AUTH FAILED (detected at Step 8)
  Wrong receiver: Gate CLOSED at Step 2
  Payload: NEVER DOWNLOADED
```

**What this means:** The full 8-step verification gate runs. Phase 1 checks the header only — no payload data touches the device. Only after all header checks pass does the payload arrive. Then its hash is verified (Step 7) and it’s decrypted (Step 8). If someone tampers with the payload, it’s caught. If the message is sent to the wrong person, the gate closes at Step 2 and the payload is never downloaded at all.

-----

## Step 5: See the Final Summary

At the bottom of the output, you’ll see:

```
==================================================================
  ALL 8 PROOFS PASSED
  Proof of Satoshi: 4 sub-scenarios verified
  Consistency check: PASS
  Tampered payload: detected
  Wrong receiver: rejected
==================================================================
  1. SHA-256 Integrity         Any modification changes the hash
  2. Bitcoin Address ID        Valid P2PKH from secp256k1 + consistency check
  3. ECDSA Signatures          Only the private key holder can sign
  4. Block Height Freshness    Stale messages rejected (anti-replay)
  5. Proof of Satoshi          Optional toggle + slider: 4 scenarios verified
  6. Address Chaining          Fresh address per message, cryptographically linked
  7. Receiver Encryption       Only the intended receiver can decrypt (ECDH)
  8. Verify-Before-Download    Gate blocks payload until all checks pass

  BUTL Protocol v1.2 — proven with zero dependencies.
  Proof of Satoshi is optional by design.
  Without PoS: 100% pure math. Zero external trust.
==================================================================
```

**If you see `ALL 8 PROOFS PASSED`, you have personally verified the protocol.**

-----

## What Just Happened (Technical Summary)

The program you ran did the following entirely on your computer, with no internet connection, no external libraries, and no pre-built cryptographic tools:

1. **Implemented the secp256k1 elliptic curve from scratch.** The same curve used by Bitcoin. Point addition, scalar multiplication, decompression — all in pure Python.
1. **Generated real Bitcoin addresses.** Not simulated. Not mocked. Real secp256k1 keypairs, real SHA-256 + RIPEMD-160 hashing, real Base58Check encoding. The addresses produced are valid Bitcoin addresses that would work on the Bitcoin network.
1. **Performed real ECDSA signature operations.** The same signature algorithm used by every Bitcoin transaction. Signing with the correct key produces a valid signature. Signing with the wrong key does not.
1. **Computed real ECDH shared secrets.** The sender multiplied their private key by the receiver’s public key. The receiver multiplied their private key by the sender’s public key. The results matched — the same shared secret, computed from different sides.
1. **Encrypted and decrypted a real message.** Using the ECDH shared secret as an encryption key, with authenticated encryption that detects any tampering.
1. **Ran the complete BUTL verification gate.** All 8 steps, in order, with correct and incorrect inputs to prove both acceptance and rejection work.

The entire file is 549 lines of Python. You can open it in any text editor and read every line. There are no hidden calls, no external servers, no tricks. The math either works or it doesn’t. It works.

-----

## Troubleshooting

### “python3: command not found”

Python isn’t installed or isn’t in your PATH. Install it from [python.org/downloads](https://www.python.org/downloads/). On Windows, make sure to check “Add Python to PATH” during installation.

### “SyntaxError: invalid syntax”

You may be running Python 2 instead of Python 3. Try `python3 butl_mvp_v12.py` explicitly (with the “3”).

### “Permission denied”

On Mac/Linux, try: `chmod +x butl_mvp_v12.py` then run again. Or run with: `python3 ./butl_mvp_v12.py`

### “No such file or directory”

You’re in the wrong folder. Make sure you `cd` to the directory where you saved the file. On Mac: `cd ~/Desktop`. On Windows: `cd %USERPROFILE%\Desktop`.

### The output looks different from this guide

The Bitcoin addresses will be different each time because they’re generated from seeds. The structure, proof names, and `ALL 8 PROOFS PASSED` message will be the same.

-----

## What to Do Next

**Want to understand the protocol?** Read [BUTL_ONE_PAGER.md](../BUTL_ONE_PAGER.md) for a 60-second overview, or [BUTL_WHITE_PAPER.md](../BUTL_WHITE_PAPER.md) for the full technical description.

**Want to build with BUTL?** See [python/README.md](../python/README.md) for the Python reference implementation or [rust/README.md](../rust/README.md) for the Rust reference implementation.

**Want to verify each proof independently?** See <VERIFICATION_CHECKLIST.md> for a printable checkbox form.

**Want a quick reference card?** See <QUICK_REFERENCE.md> for a one-page command summary.

**Want to verify the test vectors?** See [TEST_VECTORS.md](../spec/TEST_VECTORS.md) for deterministic values that any correct implementation will produce.

-----

## The Rust Version

If you have Rust installed, you can also prove the protocol with real secp256k1 + AES-256-GCM (production-quality cryptographic libraries):

```bash
cargo new butl-mvp && cd butl-mvp
# Copy butl_mvp_v12.rs to src/main.rs
# Copy Cargo.toml from the mvp/ folder
cargo run --release
```

The Rust MVP proves the same 8 claims with the same 4 Proof of Satoshi scenarios, using the same C library (libsecp256k1) that secures the Bitcoin network.

-----

## Why This Matters

Most protocols ask you to trust the authors. BUTL asks you to verify the math.

The proof you just ran doesn’t depend on Anthropic, on GitHub, on Bitcoin, or on anyone’s reputation. It depends on three mathematical facts:

1. **SHA-256 is collision-resistant.** Two different inputs produce different outputs.
1. **The secp256k1 discrete logarithm problem is hard.** Knowing the public key doesn’t reveal the private key.
1. **ECDH is commutative.** `(a × G) × b == (b × G) × a` on an elliptic curve.

If those three facts are true — and the global cryptographic community has scrutinized them for decades — then BUTL works. You just proved it does.

-----

*One file. One command. Eight proofs. The protocol works. You verified it yourself.*