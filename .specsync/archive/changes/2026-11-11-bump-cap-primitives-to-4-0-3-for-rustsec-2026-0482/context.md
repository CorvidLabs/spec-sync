---
change: bump-cap-primitives-to-4-0-3-for-rustsec-2026-0482
artifact: context
---

# Context

RUSTSEC-2026-0482 (CVSS 9): cap-primitives < 4.0.3 lets a sandboxed path ending in a slash escape through a symlink. SpecSync uses cap-std for confined spec/source reads (src/validator/consumed_by_re.rs); a hostile repo could plant a symlink plus a trailing-slash spec path to read outside the project. Fixed upstream in 4.0.3; the bump changes nothing else in Cargo.lock.
