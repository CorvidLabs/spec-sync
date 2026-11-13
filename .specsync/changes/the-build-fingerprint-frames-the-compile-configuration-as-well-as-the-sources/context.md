---
change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
artifact: context
---

# Context

#981, a Codex P2 on #920. `build.rs` hashed only file names and contents (`src/`, `Cargo.toml`,
`Cargo.lock`, `build.rs`), so builds of one tree for another target, profile or feature set shared
`SPECSYNC_BUILD_FINGERPRINT`. They could then replay each other's `.specsync/hashes.json` snapshot
or Git-common-directory read cache, for example when Windows and WSL share a checkout, although
the crate has `cfg(windows)`, `cfg(unix)` and `cfg(debug_assertions)` paths. Leif approved option
1 on 2026-11-13 ("#981 cache per build", coordinator chat 04:55Z, relayed by claude-code-t370).

`src/build_fingerprint.rs` picks the configuration variables: TARGET, PROFILE, OPT_LEVEL, DEBUG
and every CARGO_CFG_* and CARGO_FEATURE_*, sorted. `build.rs` includes it with `#[path]` and
frames them into the hasher after the files. It also records them as
`SPECSYNC_BUILD_CONFIGURATION`, so a test can see what was framed. The crate compiles the module
only for tests. The hash_cache spec gains the file and an updated `BUILD_FINGERPRINT` description.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
