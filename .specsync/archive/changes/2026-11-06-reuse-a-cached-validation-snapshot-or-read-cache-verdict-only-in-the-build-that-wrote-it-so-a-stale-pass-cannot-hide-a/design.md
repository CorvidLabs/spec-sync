---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: design
---

# Design

- `build.rs` (new) computes a fingerprint and compiles it in as `SPECSYNC_BUILD_FINGERPRINT`.
  - It is SHA-256 over every file under `src/`, plus `Cargo.toml`, `Cargo.lock` and `build.rs`, each framed by its relative path and length and sorted by path.
  - It reruns when `src/` or any of those files changes. It uses `sha2`, already a dependency, as a build-dependency.
- `hash_cache::BUILD_FINGERPRINT` (pub(crate)) holds the value. `validation_input_digest` frames it after `specsync_version`, so a snapshot from any other build, or from before #878, no longer matches and is re-validated.
  - Snapshot and cache schemas are unchanged; a stale snapshot is simply not replayed, which is the conservative failure.
- `read_cache_build_identity` becomes `CARGO_PKG_VERSION + BUILD_FINGERPRINT`. The fingerprint already covers `src/lifecycle-validation-limits.json`, so the separate limits digest becomes redundant and is replaced.
  - A cache under another identity is not loaded, so its passing verdicts are recomputed live. Refusals are never cached.
- `Cargo.toml` `include` gains `/build.rs`, so the published crate still builds.
- Rejected: the explicit `VALIDATION_RULES_VERSION` the issue suggests. A rule added without a bump repeats #878.
- Rejected: hashing the running executable. That costs I/O on every check and fails where `current_exe` is unavailable.
