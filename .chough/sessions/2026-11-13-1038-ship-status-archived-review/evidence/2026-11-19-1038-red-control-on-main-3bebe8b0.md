---
kind: evidence
id: 2026-11-19-1038-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the new tests, an_archive_tip_that_removes_its_own_package_files_is_archive_only fails (tip class product, not archive_only) and an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review fails (review_currency stale, and the stage says to re-record the review). These are the defects #1038 reports, fixed by this change; not new bugs."
result: failed
evidence_kind: product-test
revision: 4d439b395c78eb20987f11815fd46ee859bde087
content: 1b34863182f9a288aa5a2899fb3dad4140972adb3abe0d9516b44c695af88683
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:1883b203597b7fe7c66bab40f8dec044c290cfa13899ae71dd7f3952e6a1b47a
external: ext-27d35a
criteria: [SHIP-8]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T04:55:54Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the new tests, an_archive_tip_that_removes_its_own_package_files_is_archive_only fails (tip class product, not archive_only) and an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review fails (review_currency stale, and the stage says to re-record the review). These are the defects #1038 reports, fixed by this change; not new bugs.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 4d439b395c78, observed 2026-11-19T04:55:54Z.
