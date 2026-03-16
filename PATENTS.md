# Patent Grant

## Additional Grant of Patent Rights

Version 1, March 2026

"Software" means the BUTL (Bitcoin Universal Trust Layer) protocol
specification, reference implementations, documentation, and all associated
files in this repository, including any future contributions.

"Contributor" means any person or entity that distributes the Software.

"Recipient" means any person or entity that receives the Software.

Subject to the terms and conditions of this grant, each Contributor hereby
grants to each Recipient a perpetual, worldwide, non-exclusive, no-charge,
royalty-free, irrevocable (except as stated below) patent license to make,
have made, use, offer to sell, sell, import, transfer, and otherwise run,
modify, and propagate the contents of the Software, where such license
applies only to those patent claims, both currently owned or controlled and
acquired in the future, licensable by such Contributor that are necessarily
infringed by the Software.

**This patent grant applies to all users regardless of whether they choose
the MIT License or Apache License 2.0.**

---

## Patent Retaliation Clause

If Recipient institutes patent litigation against any entity (including a
cross-claim or counterclaim in a lawsuit) alleging that the Software or any
contribution within the Software constitutes direct or contributory patent
infringement, then any patent licenses granted to Recipient under this grant
for the Software shall terminate as of the date such litigation is filed.

---

## Scope

This patent grant covers all methods, processes, algorithms, data structures,
protocols, and techniques described in or implemented by the Software,
including but not limited to:

- The BUTL header format and field definitions
- The canonical signing payload construction
- The 8-step verification sequence
- The verify-before-download gate mechanism
- The address chaining and chain proof mechanism
- The use of Bitcoin block height for message freshness
- The use of Bitcoin address balance for sybil resistance (Proof of Satoshi)
- The use of ECDH with Bitcoin keys for payload encryption
- The two-phase delivery model (header verification, then payload)
- The `.well-known/butl.json` key discovery convention
- Any combination of the above

---

## Irrevocability

This grant is irrevocable. Once the Software has been published, this patent
grant cannot be withdrawn, modified, or restricted by any current or future
copyright holder, contributor, maintainer, or assignee of the Software.

---

## Prior Art Declaration

The methods described in this Software were first publicly disclosed on the
date of the initial commit to this repository. This publication constitutes
prior art under 35 U.S.C. Section 102 (United States), Article 54 EPC
(European Patent Convention), and equivalent provisions in all jurisdictions.
Any patent application filed after this date that claims the methods described
herein is anticipated by this prior art.
