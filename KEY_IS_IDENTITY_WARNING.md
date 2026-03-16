# Your Key Is Your Identity

## Read This Before You Do Anything Else

-----

## The One Rule

**Back up your private key immediately after creating it. Before sending a message. Before adding a contact. Before doing anything else.**

Everything in BUTL flows from the private key. The Bitcoin address, the public key, the ability to sign messages, the ability to decrypt messages, the entire address chain — all of it is derived from or depends on one 32-byte number.

If that number is lost, the identity is gone. Permanently. There is no recovery mechanism. There is no reset. There is no customer support. There is no one to call.

This is not a limitation that can be fixed in a future version. It is a mathematical property of the system. The same property that makes it impossible for anyone to steal, revoke, or impersonate the identity also makes it impossible for anyone to recover it. These two properties are inseparable. One cannot exist without the other.

-----

## What “Lost” Means

The private key is lost if any of the following are true:

- The file containing the private key was deleted
- The device storing the key was lost, stolen, destroyed, or factory-reset
- The password encrypting the key file was forgotten, with no backup of the unencrypted key or seed phrase
- A disk failure corrupted the file and no backup exists
- The key was never backed up and the original storage is gone
- Malware exfiltrated the key and then deleted the local copy

If the key is lost, the following are true — immediately, permanently, and without exception:

- The identity is gone. No one can prove they are that identity.
- No messages encrypted to that public key can ever be decrypted.
- No address chains started by that identity can be continued. Contacts will see a broken chain.
- No new messages can be signed as that identity.
- No one can help. Not the BUTL developers. Not GitHub. Not the Bitcoin network. Not anyone on earth.

-----

## What “Compromised” Means

The private key is compromised if anyone other than the rightful holder has obtained a copy. This can happen through:

- Malware that reads the key from disk or memory
- A stolen device where the key file was not encrypted (or the encryption was weak)
- An insecure backup that was accessed by someone else
- Sharing the key intentionally or accidentally
- A compromised cloud storage service where an unencrypted backup was stored

If the key is compromised, the following are true:

- Someone else can sign messages as that identity. Recipients will believe those messages are authentic.
- Someone else can decrypt messages that were encrypted to that public key.
- Someone else can continue the address chain, maintaining the illusion that they are the original sender.
- There is currently no protocol-level mechanism to revoke the compromised key. Contacts must be notified through a separate channel to stop trusting that identity.

-----

## How to Protect the Key

### 1. Back Up Immediately After Creation

The moment a BUTL identity is generated, back up the private key to at least two physically separate locations. Do this before sending a message, before adding a contact, before anything.

**Good backup methods:**

- **BIP-39 seed phrase (recommended).** Convert the master seed to a 12 or 24-word mnemonic. Write the words on paper. Store the paper in a safe, fireproof location. The same words will regenerate the exact same identity on any device, in any implementation, forever.
- **Encrypted file on a USB drive.** Export the encrypted key file to a USB drive. Store the drive in a physically secure location separate from the primary device.
- **Paper backup of raw hex.** Print or write the 64-character hex string of the private key. Store it in a safe. This is a last resort — seed phrases are easier to write correctly and verify.

**Bad backup methods:**

- The same device that holds the original. If the device dies, both copies die.
- An unencrypted file on cloud storage. Anyone who accesses the account has the identity.
- A screenshot on a phone. Phones are lost, stolen, and compromised constantly.
- Email. Email accounts are breached regularly.
- A text message or chat. These are stored on servers controlled by others.

### 2. Encrypt the Key at Rest

Never store the private key as a plaintext file. Always encrypt it with a strong password.

If someone gains access to the device — through theft, malware, or unauthorized access — and the key file is unencrypted, they have the identity. They can sign messages, decrypt messages, and extend the address chain. They are cryptographically indistinguishable from the rightful holder.

Recommended encryption: Argon2id (or PBKDF2 with 100,000+ iterations) for password-to-key derivation, AES-256-GCM for file encryption. The BUTL reference implementations and applications should handle this automatically.

### 3. Use a Seed Phrase

For the best balance of security and recoverability, derive the BUTL identity from a BIP-39 mnemonic seed phrase — the same 12 or 24-word phrase used by Bitcoin wallets.

Write the words on paper. Store the paper in a safe. If the device is lost, the exact identity can be regenerated from the words. If the paper is lost but the device still works, the encrypted key file on disk still works. Two independent recovery paths.

The seed phrase IS the identity. Anyone with the words becomes that identity. Guard them accordingly.

### 4. Never Share the Private Key

The private key should never leave the device except in the form of an encrypted backup under the holder’s control.

Never paste it into a website. Never email it. Never type it into a chat. Never show it on a screen share. Never photograph it in an insecure location. Never speak it aloud in a public place.

If anyone asks for a private key — for any reason — the answer is always no. No legitimate BUTL application will ever request a private key. They use it internally and never expose it.

The **public key** and **Bitcoin address** can and should be shared freely. They are designed to be public. They cannot be used to derive the private key.

### 5. Plan for Device Loss

Assume the device will eventually be lost, stolen, broken, or replaced. This is not pessimism — it is statistical certainty over a long enough timeline.

If the backup strategy from steps 1-3 is in place, device loss is an inconvenience — restore the seed phrase on a new device, regenerate the identity, and continue. If the backup strategy is not in place, device loss is permanent identity loss.

-----

## What to Do If the Key Is Lost

1. **Accept that the old identity is gone.** There is no recovery path. This is final.
1. **Generate a new BUTL identity.** A new keypair produces a new address and new public key.
1. **Notify contacts.** Tell them the old address is no longer valid. Provide the new public key through a trusted channel.
1. **Start a new address chain.** The new identity begins a new chain (genesis message). There is no cryptographic link between the old and new identities because the old private key no longer exists.
1. **Back up the new key immediately.** Do not repeat the mistake.

-----

## What to Do If the Key Is Compromised

1. **Stop using that identity immediately.** Any message signed with that key could be forged by the attacker.
1. **Notify contacts through a separate channel.** Phone call, in-person conversation, verified social media — anything other than the compromised BUTL identity. Tell them to reject all messages from that address chain.
1. **Generate a new BUTL identity.** Distribute the new public key through the same trusted channels.
1. **Investigate the compromise.** Determine how the key was obtained and close the vulnerability (change passwords, remove malware, encrypt backups, replace hardware).
1. **Understand the limitations.** The attacker still has the old key. They can still sign messages as the old identity and extend the old address chain. The only defense is contacts manually stopping trust in the old ThreadID. Protocol-level revocation is planned for a future version.

-----

## For Developers Building BUTL Applications

Applications that manage BUTL private keys carry a serious responsibility. The following are requirements, not suggestions:

**Encrypt keys at rest.** Use the operating system’s keychain (macOS Keychain, Windows DPAPI, Linux secret-service) or password-derived encryption (Argon2id + AES-256-GCM). Never store a private key in plaintext.

**Never log private keys.** Not in debug logs. Not in crash reports. Not in analytics. Not in error messages. Not in database fields. Not anywhere. Review log output explicitly to confirm no key material leaks.

**Never include private keys in handoffs.** The BUTL handoff protocol passes a reference to the encrypted key file, not the key itself. Spoke applications request signing operations from BUTL-ID; they never hold the raw key.

**Prompt for backup immediately.** After key generation, the application should display the private key (or seed phrase) once, clearly, with unmistakable warnings. The user should not be able to proceed until they confirm they have backed up the key.

**Zero key memory after use.** When a private key is loaded into memory for a signing or ECDH operation, the memory must be explicitly zeroed after the operation completes. In Rust, use the `zeroize` crate. In Python, use `ctypes.memset` or a secure memory library. Do not rely on garbage collection.

**Never build “recovery” features that store the key remotely.** If an application stores a copy of the private key on a server “for recovery,” it has created a centralized point of failure, a honeypot for attackers, and has defeated the fundamental purpose of self-sovereign identity. The private key must never exist anywhere the holder does not control.

**Show clear warnings for irreversible actions.** Before deleting a key file, resetting the application, or any action that could result in key loss, display an explicit warning and require confirmation.

-----

## The Tradeoff

Every trust system makes a tradeoff between sovereignty and recoverability.

Passwords can be reset because a service holds the real credential. But that service can be hacked, can lock the user out, or can be compelled to give access to others.

BUTL keys cannot be reset because no one else holds them. But if the key is lost, no one can bring it back.

This is the same tradeoff Bitcoin makes. The same tradeoff that makes self-custody powerful and permanent. The same tradeoff that has existed since the invention of public-key cryptography in 1976.

Choosing BUTL is choosing sovereignty over convenience. That is a serious choice. Treat it seriously.

-----

## Summary

|                    |Key Protected|Key Lost          |Key Compromised          |
|--------------------|-------------|------------------|-------------------------|
|**Sign messages**   |Yes          |No — permanently  |Yes — by the attacker    |
|**Decrypt messages**|Yes          |No — permanently  |Yes — by the attacker    |
|**Continue chain**  |Yes          |No — chain breaks |Yes — attacker can extend|
|**Prove identity**  |Yes          |No — identity gone|Ambiguous — two holders  |
|**Recovery path**   |N/A          |None              |Generate new identity    |

-----

**Back up the key. Today. Right now. Before anything else.**

-----

*Your key is your identity. Protect it like there is no second chance — because there isn’t.*