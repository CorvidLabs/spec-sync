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

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-hash-cache-005 | unit tests a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal and the another-build test; snapshot-digest mutant; repro-878.sh end to end |
| REQ-change-134 | unit test a_read_cache_written_by_a_build_before_878_is_not_reused; read-cache identity mutant |
