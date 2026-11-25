---
change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
artifact: testing
---

# Testing

- Integration, in `tests/integration/check.rs`, each failing before its fix: `a_cached_check_reports_a_3md_broken_since_the_last_run`, `the_3md_checked_is_named_for_the_module_not_the_spec_file`, `a_directory_named_as_the_3md_is_reported`.
- Unit, in `src/hash_cache.rs`: `a_snapshot_is_not_replayed_over_a_changed_lesson_file`.
- The four findings re-run on the CLI with the binary the native lane builds, against the binary of #1163's head.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-023 | `tests/integration/check.rs`: `an_unparseable_3md_is_a_check_warning_and_fails_strict`, `a_3md_over_the_read_bound_is_reported_and_not_read`, `the_3md_checked_is_named_for_the_module_not_the_spec_file`, `a_directory_named_as_the_3md_is_reported`, `a_cached_check_reports_a_3md_broken_since_the_last_run` |
| REQ-hash-cache-011 | `src/hash_cache.rs`: `a_snapshot_is_not_replayed_over_a_changed_lesson_file`; `tests/integration/check.rs`: `a_cached_check_reports_a_3md_broken_since_the_last_run` |
