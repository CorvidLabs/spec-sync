# Lesson bundle: refresh-url-lockfile-for-rustsec-2026-0592

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refresh url lockfile for RUSTSEC-2026-0592
- **Kind**: BugFix
- **Paths**: Cargo.lock
- **Acceptance**: Cargo.lock resolves url 2.5.9 and its required form_urlencoded 1.2.3 with no other dependency changes.
- **Acceptance**: Cargo audit clears RUSTSEC-2026-0592 and the full project verification lane passes on pinned Rust.

## Evidence

- Verification commit: `786f9362098e49a98e15b436a80154c0b61c3373`
- Base commit: `ba729a08dde0b269518fb9a5b9037155d1febcd3`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

GitHub Actions rejected PR #1144 in the `audit` job because the unchanged main lockfile contains `url` 2.5.8. RustSec advisory [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592.html) lists 2.5.9 as the first fixed release. The lockfile records `cookie_store` 0.22.0 as the immediate dependent.

Cargo 1.89's offline dry run resolves `url` 2.5.9 and `form_urlencoded` 1.2.3. Registry metadata gives `url` 2.5.9 a Rust 1.63 minimum and requires `form_urlencoded` ^1.2.3. The product path in scope is `Cargo.lock`.

## From the change's testing.md

# Testing

- Run `cargo update -p url --precise 2.5.9` using the pinned Rust toolchain. Inspect the product diff and verify that its lockfile entries change for `url` and `form_urlencoded`.
- Run `cargo audit` against the current RustSec database and confirm that RUSTSEC-2026-0592 clears.
- Run `fledge lanes run verify`, SpecSync strict validation with full coverage, `specsync change audit --strict`, `fledge lanes run pre-push`, and `fledge trust verify` against the candidate before the scoped human review.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
