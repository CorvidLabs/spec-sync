# Lesson bundle: bump-cap-primitives-to-4-0-3-for-rustsec-2026-0482

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Bump cap-primitives to 4.0.3 for RUSTSEC-2026-0482
- **Kind**: BugFix
- **Paths**: Cargo.lock
- **Acceptance**: cargo audit no longer reports RUSTSEC-2026-0482; the full suite passes unchanged

## Evidence

- Verification commit: `9f85c34f3fe08ea4e28074c228d6003725e2c295`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

RUSTSEC-2026-0482 (CVSS 9): cap-primitives < 4.0.3 lets a sandboxed path ending in a slash escape through a symlink. SpecSync uses cap-std for confined spec/source reads (src/validator/consumed_by_re.rs); a hostile repo could plant a symlink plus a trailing-slash spec path to read outside the project. Fixed upstream in 4.0.3; the bump changes nothing else in Cargo.lock.

## From the change's testing.md

# Testing

Existing suite plus cargo audit.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
