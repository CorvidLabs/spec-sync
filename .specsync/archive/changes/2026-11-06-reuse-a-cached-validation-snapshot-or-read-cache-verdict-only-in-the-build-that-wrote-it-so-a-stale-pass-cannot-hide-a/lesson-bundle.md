# Lesson bundle: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Reuse a cached validation snapshot or read-cache verdict only in the build that wrote it, so a stale pass cannot hide a new refusal (#878)
- **Kind**: BugFix
- **Specs**: hash_cache, change
- **Paths**: build.rs, Cargo.toml, Cargo.lock, src/hash_cache.rs, src/change/read_cache.rs, src/change_tests/fast_archived_reads.rs, CHANGELOG.md, specs/hash_cache/, specs/change/, hi/check.md, INTENT.md, .chough/
- **Acceptance**: A validation snapshot recorded by any other build, including one that reports the same package version and one written before #878, is not replayed: a default specsync check re-validates the spec and reports the refusal this build makes (a duplicated requirement ID in the #878 case), while a snapshot recorded by this build still replays.
- **Acceptance**: The persistent read cache written by another build of the same version, such as a sibling worktree sharing the Git common directory, is not loaded; its passing verdicts are recomputed live.
- **Acceptance**: Both bindings come from one build fingerprint compiled in by build.rs over src/, Cargo.toml, Cargo.lock and build.rs; dropping it from either digest fails a test (mutation), and the end-to-end repro (released 6.2.0 then this build) reports the duplicate instead of All specs unchanged.

## Evidence

- Verification commit: `c3d15857c0a21811f69314f866d6c3f47013b49e`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec change --spec hash_cache`

## From the change's context.md

# Context

- Issue #878 was filed from the #865 review. The duplicate-ID refusal runs in per-spec validation, but the snapshot digest keys only on the version and the format versions. A snapshot from an earlier 6.2.0 build can therefore replay a pass in a default `specsync check` until a file changes.
- Plan row S6 (c). Leif approved CHECK-1.d and approach B (build fingerprint) at 20:20 MST, then "Both in one" at 22:10 MST, so the persistent read cache, which has the same flaw, is fixed here too. Relayed by the orc (t18).
- Human gates: scope approval and implementation review come from Leif through the orc. This agent never records them as Leif.

## From the change's design.md

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

## From the change's testing.md

# Testing

## Plan

- `a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal` (unit, src/hash_cache.rs) forges a clean snapshot under the frozen pre-#878 digest for a spec whose `requirements.md` repeats an ID. The snapshot must not replay, and this build reports the duplicate.
- `a_read_cache_written_by_a_build_before_878_is_not_reused` (unit, src/change_tests/fast_archived_reads.rs) stamps a cache holding a passing verdict with the pre-#878 identity. The next read must recompute and rewrite it under this build's identity.
- A second hash_cache test covers a snapshot from another fingerprint at the same version (not replayed) and the same clean snapshot from this build (replayed: the control).
- Mutants:
  - `build_fingerprint` frame removed from the snapshot digest;
  - fingerprint removed from the read-cache identity.
  Each must fail its test, and each source is restored byte for byte.
- End to end, `repro-878.sh`: released 6.2.0, then the fixed build, on the same unchanged files. The default check must exit 1 and name `REQ-auth-001`.
- `fledge lanes run verify` (pinned 1.89) and pre-push.

## Results (local macOS, pinned rustc 1.89.0)

- Red first, on main cd8cab5e plus the two new tests only. `a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal` failed: the pre-#878 snapshot replayed. `a_read_cache_written_by_a_build_before_878_is_not_reused` failed: the pre-#878 cache was reused.
- Green at 852ba37f: both, plus `only_the_build_that_recorded_a_snapshot_replays_it` (another fingerprint is not replayed; this build's is). 64 unit tests matching hash_cache, archived and cache filters, and 70 integration tests matching check, staleness and cache filters, pass.
- End to end with `repro-878.sh`: released 6.2.0 records a clean snapshot; the fixed build's default `specsync check` then reports `REQ-auth-001` appears 2 times and exits 1. Before the fix it said "All specs unchanged" and exited 0. On a clean spec, a second run of the fixed build still skips it ("Skipped 1 unchanged spec(s)"), so the cache still works within one build.
- Mutants at 852ba37f, each restored byte for byte:

| Mutant | Change | Where the test fails |
| --- | --- | --- |
| M878-snapshot | the snapshot digest drops the build fingerprint | `a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal`: the pre-#878 snapshot replays |
| M878-snapshot-other-build | same mutation | `only_the_build_that_recorded_a_snapshot_replays_it`: another build's snapshot replays |
| M878-read-cache | the read-cache identity is version plus limits digest again | `a_read_cache_written_by_a_build_before_878_is_not_reused`: the old cache is reused |

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-hash-cache-005 | unit tests a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal and the another-build test; snapshot-digest mutant; repro-878.sh end to end |
| REQ-change-134 | unit test a_read_cache_written_by_a_build_before_878_is_not_reused; read-cache identity mutant |

## Where these lessons go

- `specs/hash_cache/hash_cache.3md`
- `specs/change/change.3md`
