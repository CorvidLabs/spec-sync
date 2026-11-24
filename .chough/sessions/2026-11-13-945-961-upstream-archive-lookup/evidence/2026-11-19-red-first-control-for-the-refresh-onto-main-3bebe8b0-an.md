---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-an
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup (#961) and an_upstream_rename_of_an_archive_head_already_contains_is_not_reported (#945) both fail, the defects the issues report."
result: failed
evidence_kind: product-test
revision: e816e662ca6499b57e5cacb0298e5f323db97872
content: eefd1f01777b5d272f6fcdc0d07cf4155c59637b94cd372fb71af18daa9d9c72
environment: "red control: main 3bebe8b0 plus only the new tests"
criteria: [SHIP-3.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T07:33:30Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup (#961) and an_upstream_rename_of_an_archive_head_already_contains_is_not_reported (#945) both fail, the defects the issues report.**

Recorded as failed in red control: main 3bebe8b0 plus only the new tests at revision e816e662ca64, observed 2026-11-19T07:33:30Z.

## Key failing lines

- 2 failed: an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup, an_upstream_rename_of_an_archive_head_already_contains_is_not_reported
