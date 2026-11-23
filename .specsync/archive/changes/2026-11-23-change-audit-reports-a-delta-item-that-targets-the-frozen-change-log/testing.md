---
change: change-audit-reports-a-delta-item-that-targets-the-frozen-change-log
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-170 | `src/change_tests/stale_modified_delta.rs`: `audit_reports_a_change_log_delta_item_of_an_active_change`, `audit_does_not_judge_an_archived_change_log_item_again`, `audit_reports_the_change_log_item_before_a_stale_base`, and the audit's strict failure |

## How it runs

Unit tests, red on main 2cc0897f and then green; the full verify lane; and `change audit --strict` on the real repositories.
