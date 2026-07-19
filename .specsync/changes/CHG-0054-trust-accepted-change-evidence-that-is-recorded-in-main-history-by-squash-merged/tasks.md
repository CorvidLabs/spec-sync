---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: tasks
---

# Tasks

- [x] Implement the recording-anchor fallback in accepted-transition authentication.
- [x] Add regression coverage for squash-merged refreshed evidence and for fail-closed behavior
  when no in-history accepted record matches.
- [x] Add and map canonical requirement REQ-change-037 and extend the canonical Invariants
  section.
- [x] Run pre-acceptance formatting, lint, unit/integration tests, and release validators.
- [x] Prepare the post-acceptance archival of CHG-0048 (musl), CHG-0051, CHG-0052, and CHG-0053
  with the fixed binary, then run forced strict and Trust.
