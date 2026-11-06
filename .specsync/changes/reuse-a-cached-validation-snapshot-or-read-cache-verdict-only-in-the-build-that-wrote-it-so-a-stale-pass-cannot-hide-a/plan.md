---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: plan
---

# Plan

## Approach

Bind both caches to one compile-time build fingerprint, so no cache entry crosses builds. The failing tests come first, and each binding is proven by a mutant.

## Out of scope

- The cache file format (`CACHE_FORMAT_VERSION`, `SNAPSHOT_VERSION`, `READ_CACHE_SCHEMA`): unchanged.
- CI behavior (`--strict` already re-validates everything).
- Other caches: none other persists results across processes.

## Steps

1. Red tests, already committed with this draft:
   - `a_clean_snapshot_written_before_878_does_not_hide_a_new_refusal` (src/hash_cache.rs) forges a clean snapshot with a frozen copy of the pre-#878 digest over a duplicated requirement ID.
   - `a_read_cache_written_by_a_build_before_878_is_not_reused` (src/change_tests/fast_archived_reads.rs) stamps the persistent cache with the pre-#878 identity.
2. `build.rs` plus `Cargo.toml` (`[build-dependencies] sha2`, `include += /build.rs`).
3. `BUILD_FINGERPRINT` in the snapshot digest; a second test for "another build at the same version" and a same-build control.
4. `read_cache_build_identity` with the fingerprint.
5. Mutants: drop the fingerprint from each binding; each kills its test.
6. Re-run the end-to-end repro: step 2 must now exit 1 and name the duplicate.
7. Specs (Public API rows, hash_cache `files: build.rs`, Change Log), CHANGELOG, and `fledge lanes run verify` (pinned 1.89).

## Risks

- Developer rebuilds invalidate caches more often: after any source edit, the first default check re-validates every spec once. Correctness over a cache hit. Release builds of one tree share the fingerprint, because the sources are identical.
- Packaging: without `/build.rs` in `include`, `cargo publish` would fail to build. The step adds it, and the release lane's package step exercises it.
- Line endings: a CRLF checkout fingerprints differently from an LF one. Harmless, since caches are local.

## Constraints consulted

- SPEED-1.c (correct invalidation rule) and CHECK-1.d.
- specs/change/change.3md: a gate that returns before evidence is written leaves the old evidence standing. Here, a stale cached pass must not stand as the answer.
- 95% plan: failing test first; never weaken a test, gate or bound.
- Leif's choice of approach B, both caches in one change.
