# Lesson bundle: refresh-cookie-lockfile-for-rustsec-2026-0594

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refresh cookie lockfile for RUSTSEC-2026-0594
- **Kind**: BugFix
- **Paths**: Cargo.lock
- **Acceptance**: Cargo.lock resolves cookie 0.18.3 instead of 0.18.1. Only the version and checksum in the existing cookie entry change.
- **Acceptance**: The resolved features and dependency graph match the baseline apart from the cookie version, and audit runs with the existing policy and clears RUSTSEC-2026-0594.
- **Acceptance**: The full project verification lane passes on the pinned Rust toolchain.

## Evidence

- Verification commit: `28e05731184c42ab4b45f991359c3ac97fb3a90d`
- Base commit: `71f1867d7ddcf2b075b49c421d2f87ea0b760be1`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

The RustSec dependency audit on main at `71f1867d` reports RUSTSEC-2026-0594 for the transitive `cookie 0.18.1`. The advisory lists `0.18.3` as a compatible patched version. An isolated preflight found that Cargo can select it with only a version and checksum change in the existing lockfile entry.

The product scope is `Cargo.lock` only. Preserve the existing manifest, resolution features, dependency edges, and audit warning policy. The patch refresh changes no SpecSync requirement or public API.

## From the change's testing.md

# Testing

After scope approval, resolve `cookie 0.18.3` with the pinned toolchain. Inspect the lockfile diff and compare the resolved dependency graph and enabled features against main, accounting for the cookie version label. Run the repository’s RustSec audit with its existing policy and confirm RUSTSEC-2026-0594 clears.

Run the repository verification and pre-push lanes, the strict lifecycle audit, and the Trust and Attest checks on the resulting candidate before asking for implementation review.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
