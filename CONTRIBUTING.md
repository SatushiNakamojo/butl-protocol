# Contributing to the BUTL Protocol

## Welcome

Thank you for your interest in contributing to the Bitcoin Universal Trust Layer. BUTL is an open protocol that belongs to everyone. Every contribution — whether it’s fixing a typo, improving documentation, writing code, or proposing a protocol change — makes the protocol stronger.

-----

## A Note on How This Project Operates

This project may have only one person maintaining it. That means there may be only one person reviewing pull requests, triaging issues, and responding to questions. Patience is appreciated, clarity is essential, and every contribution is genuinely valued.

Response times may vary from hours to days to occasionally longer. A delay should not be interpreted as disinterest — it usually means the maintainer is working through a backlog or thinking carefully about a contribution. If something is urgent (especially security-related), note it clearly in the title. Security reports always get priority.

If a pull request has not received a response in two weeks, a polite follow-up comment is always welcome.

-----

## Legal Agreement

### By contributing, you agree to the following:

**1. Dual License.** Your contribution is licensed under both the **MIT License** (<LICENSE-MIT.md>) and the **Apache License 2.0** (<LICENSE-APACHE.md>). Users of the BUTL protocol may choose either license, at their option.

**2. Patent Grant.** You grant the patent rights described in <PATENTS.md> for your contribution. This is a perpetual, worldwide, royalty-free, irrevocable patent license covering the methods in your contribution.

**3. Defensive Patent Pledge.** You agree to the [Defensive Patent Pledge](legal/DEFENSIVE-PATENT-PLEDGE.md). You will not file patents on BUTL methods and will assist in defending against patent trolls.

**4. Right to Contribute.** You represent that you have the legal right to make these grants and that your contribution does not infringe any third-party rights you are not willing to license under these terms.

**5. Prior Art.** You understand that your contribution, once merged, becomes part of the public record and establishes prior art that prevents anyone from patenting the contributed methods.

These terms exist to keep BUTL permanently free and open. They protect every contributor, every user, and the protocol itself.

-----

## Ways to Contribute

### Report a Bug

Found something broken? Open a [GitHub Issue](../../issues/new) with:

- A clear, descriptive title
- Steps to reproduce the problem
- What was expected to happen vs what actually happened
- Environment details (OS, Python/Rust version, library versions)

### Suggest a Feature

Have an idea? Open a [GitHub Issue](../../issues/new) with:

- What the feature would do
- Why it matters (what problem does it solve?)
- Whether it affects the protocol specification or only implementations

### Improve Documentation

This is one of the most impactful contributions and does not require deep protocol expertise. If something was confusing to read, fixing it helps the next person. This includes fixing typos, adding examples, improving clarity, translating to other languages, and expanding the FAQ.

### Write Code

Code contributions can include bug fixes, new test cases, performance improvements, new language implementations (JavaScript, Go, C, etc.), tools, and application integrations.

### Propose Protocol Changes

Changes to the protocol specification (`spec/`) require the most careful consideration. See the “Protocol Changes” section below.

-----

## How to Submit a Contribution

### Step 1: Fork the Repository

Click the “Fork” button on the top right of the repository page.

### Step 2: Create a Branch

```bash
git clone https://github.com/YOUR_USERNAME/butl-protocol.git
cd butl-protocol
git checkout -b your-branch-name
```

Use a descriptive branch name: `fix/typo-in-white-paper`, `feature/javascript-implementation`, `docs/add-websocket-example`.

### Step 3: Make the Changes

Edit, write, test. Quality matters more than speed.

### Step 4: Test

**Python code:**

```bash
python3 mvp/butl_mvp_v12.py       # Must still show "ALL 8 PROOFS PASSED"
python3 python/butl_v12.py          # Reference implementation demo must run
```

**Rust code:**

```bash
cargo test
cargo clippy                        # No warnings
cargo fmt -- --check                # Code must be formatted
```

**Documentation:** Read the changes as if seeing the project for the first time. Check that links work. Verify that code examples are accurate.

### Step 5: Commit

Write a clear commit message:

```bash
git commit -m "Fix ECDH shared secret computation for 0x03 prefix keys

The previous implementation incorrectly handled y-coordinate
decompression for odd-prefix keys. Added test case for both
even and odd y prefixes.

Fixes #42"
```

A good commit message has a short summary line (under 72 characters), a blank line, and then the explanation of what changed and why.

### Step 6: Push and Open a Pull Request

```bash
git push origin your-branch-name
```

Go to the fork on GitHub, click “Compare & pull request,” and fill in:

- **Title:** Clear, concise description
- **Description:** What changed, why, and how to test it

### Step 7: Wait for Review

Since this project may only have one reviewer, pull requests are reviewed as quickly as possible but timelines may vary. The review may result in an approval, a request for changes, or questions to better understand the approach. This is a collaborative process, not an exam — the goal is to make the contribution as good as possible.

-----

## Protocol Changes

Changes to the protocol specification (`spec/`) affect every implementation and every user, so they require extra care.

### What’s Needed

1. **A Proposal.** Open a GitHub Issue labeled `protocol-change` with the problem, proposed solution, backward compatibility analysis, and security implications.
1. **Discussion.** The proposal will be reviewed and discussed in the issue thread. Protocol changes are not rushed.
1. **Implementation.** If accepted in principle, the change needs a reference implementation in at least one language (Python or Rust, ideally both).
1. **Updated Tests.** New or modified features need updated MVP proofs or test vectors.
1. **Updated Documentation.** The specification, header registry, and any affected guides must be updated.

### What Requires a Version Bump

A new protocol version (`BUTL-Version: 2`) is required when the canonical signing payload format changes, a required header field is added or removed, the verification step order changes, or the encryption scheme changes incompatibly.

A version bump is NOT required when an optional header field is added, documentation is updated, or recommended thresholds change.

-----

## Code Style

### Python

- [PEP 8](https://peps.python.org/pep-0008/)
- Type hints for function signatures
- Maximum line length: 100 characters
- Docstrings for public functions and classes
- `snake_case` for functions and variables, `PascalCase` for classes

### Rust

- `cargo fmt` before committing
- `cargo clippy` with no warnings
- Doc comments (`///`) for public items
- Return `Result` over panicking

### Documentation

- Markdown (`.md`)
- Clear and concise
- Plain language — many readers are not native English speakers
- Code examples that actually work

-----

## What Will Not Be Merged

To protect the protocol and its users:

- **Changes that weaken legal protections.** Any modification to LICENSE files, PATENTS, or the Defensive Patent Pledge that reduces user protections.
- **New trust dependencies in the core protocol.** BUTL without Proof of Satoshi is pure math. That property is non-negotiable.
- **Backdoors or intentional vulnerabilities.**
- **Contributions with incompatible licenses.** Code from GPL/AGPL projects cannot be merged because it conflicts with the MIT license option.
- **Submissions without tests or documentation.** If it’s code, it needs tests. If it changes behavior, it needs documentation.

-----

## Labels

|Label             |Meaning                              |
|------------------|-------------------------------------|
|`bug`             |Something is broken                  |
|`enhancement`     |New feature or improvement           |
|`documentation`   |Documentation change                 |
|`protocol-change` |Affects the specification            |
|`good first issue`|Good entry point for new contributors|
|`question`        |Discussion or clarification          |
|`security`        |Security-related                     |
|`help wanted`     |Assistance is needed on this         |

-----

## Recognition

Every contributor is recognized. Significant contributors will be added to `CONTRIBUTORS.md`. Every merged pull request is a permanent part of the project’s history.

-----

## Questions?

If there is uncertainty about anything — whether an idea fits, how to structure a contribution, or what the process looks like — open an issue and ask. There are no stupid questions. A shaped contribution is better than no contribution at all.

-----

*BUTL is an open protocol. Every contribution makes it stronger. Thank you.*