---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: testing
---

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
