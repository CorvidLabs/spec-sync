---
change: check-and-change-audit-scan-a-legacy-module-req-md-for-duplicate-requirement-ids
artifact: testing
---

# Testing

- Unit `a_legacy_req_md_companion_is_scanned_for_duplicate_ids` in
  `src/validator/requirement_ids.rs` (the audit's scan; fails on main: no error).
- Integration `check_refuses_a_duplicate_requirement_id_in_a_legacy_req_md` in
  `tests/integration/check.rs` (fails on main: `check` passes).
- The existing duplicate-ID tests still pass.
