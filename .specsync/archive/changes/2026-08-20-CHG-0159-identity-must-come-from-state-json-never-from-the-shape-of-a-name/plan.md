# Plan

1. `src/change.rs` — add `name_carries_a_lifecycle_ordinal` and `directory_holds_a_regular_file`;
   rewrite `is_positive_legacy_tombstone` as a union of the three signals.
2. `src/change_tests.rs` — two discriminating tests plus a vacuity control.
3. `.github/scripts/classify-ci-paths.sh` — read `.id` from archived state in
   `record_archive_path`; glob `*/state.json`; take `([^/]+)` in the two review-path patterns.
4. `.github/scripts/test-classify-ci-paths.sh` — give both fixture sets an `id`, and add a
   slug-only change that must still require review.
5. Discriminate every assertion against a separate checkout of `origin/main`.
