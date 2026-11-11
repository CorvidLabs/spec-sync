# Lesson bundle: minimal-cap-primitives-bump-audit-in-evidence

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Minimal cap-primitives bump, audit in evidence
- **Kind**: BugFix
- **Paths**: Cargo.lock
- **Acceptance**: Cargo.lock's diff vs main is exactly cap-primitives' version and checksum; cargo audit exits 0 at the head; tasks.md ticks only completed items

## Evidence

- Verification commit: `a13c8bf4876c5698dea62286c38ce3c7a85a2098`
- Base commit: `9f85c34f3fe08ea4e28074c228d6003725e2c295`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

corvid-agent+t370 on #1054: cargo update re-resolved 9 crates windows-sys edges to 0.52.0 (they are not yanked; any cargo touch re-normalizes them), so the minimal change is a hand edit of the two cap-primitives lines, verified with cargo build --locked. cargo audit output is captured as evidence. tasks.md ticks only what has run.

## From the change's testing.md

# Testing

cargo audit exit 0 (no RUSTSEC-2026-0482); cargo build --locked exit 0; full suite under the shared lock.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
