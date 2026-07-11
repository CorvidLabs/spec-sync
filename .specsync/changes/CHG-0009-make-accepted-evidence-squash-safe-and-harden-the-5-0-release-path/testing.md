---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: testing
---

# Testing

- Reproduce a feature branch acceptance followed by a squash merge onto the remote default branch.
- Prove unchanged accepted evidence passes and archives.
- Prove an unintegrated branch and changed scoped input both fail.
- Run `cargo test`, strict SpecSync checks, release packaging dry-run, site checks/build, and clean Action consumers.
