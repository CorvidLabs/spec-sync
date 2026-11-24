---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-all-five
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: all five literal_guidance tests fail (glob characters in the restore, the archiving-commit lookup, the package probe, the leading-hyphen id, a control byte in a directory name), and so do the four updated unit tests (strict_check_reports_standalone_unprovable_archived_history, handoff_corrupt_archive_is_not_yet_and_names_the_restore, corrupt_archive_restore_quotes_every_interpolated_word, corrupt_archive_restore_names_the_package_only_when_the_package_changed) and the integration test a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it. These are the defects #929, #930 and #931 report."
result: failed
evidence_kind: product-test
revision: 17e42ce125145d2d736413a627c0a37bf83bc6b6
content: ef97c85dde273b3b2a9bef64dc7fad8a4b9247e049e26b50cfb09f545d497ad7
environment: "red control: main 3bebe8b0 plus #929's behavior-neutral refactor 247b6d2a (it names archive_package_differs_from, which a new test calls) plus only the new and changed tests"
criteria: [CHANGE-7.a.2, CHANGE-7.b]
requirements: [REQ-change-093]
contracts: []
limitations: []
observed: 2026-11-19T07:18:34Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: all five literal_guidance tests fail (glob characters in the restore, the archiving-commit lookup, the package probe, the leading-hyphen id, a control byte in a directory name), and so do the four updated unit tests (strict_check_reports_standalone_unprovable_archived_history, handoff_corrupt_archive_is_not_yet_and_names_the_restore, corrupt_archive_restore_quotes_every_interpolated_word, corrupt_archive_restore_names_the_package_only_when_the_package_changed) and the integration test a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it. These are the defects #929, #930 and #931 report.**

Recorded as failed in red control: main 3bebe8b0 plus #929's behavior-neutral refactor 247b6d2a (it names archive_package_differs_from, which a new test calls) plus only the new and changed tests at revision 17e42ce12514, observed 2026-11-19T07:18:34Z.

## Key failing lines

- the_printed_archiving_commit_lookup_matches_only_the_named_package: left 3f9a7c21, right 12bd3e7e
