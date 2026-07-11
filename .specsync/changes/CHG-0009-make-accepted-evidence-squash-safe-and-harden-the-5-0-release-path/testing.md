---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: testing
---

# Testing

- Covers `REQ-change-016`.
- Reproduce a feature branch acceptance followed by a squash merge onto the remote default branch.
- Prove unchanged accepted evidence passes and archives.
- Prove an unintegrated branch and changed scoped input both fail.
- Run `cargo test`, strict SpecSync checks, release packaging dry-run, site checks/build, and clean Action consumers.

## Results

- Rust: 1,514 unit and 187 integration tests passed.
- Formatting and Clippy with warnings denied passed.
- Strict lifecycle/spec validation passed: 62 specs, zero warnings/errors, 100% file and LOC coverage.
- The merged-tree archive migration succeeded for CHG-0001, 0002, 0004, 0006, 0007, and 0008.
