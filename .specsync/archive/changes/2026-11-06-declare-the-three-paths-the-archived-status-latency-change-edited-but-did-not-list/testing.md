---
change: declare-the-three-paths-the-archived-status-latency-change-edited-but-did-not-list
artifact: testing
---

# Testing

- `specsync change audit --strict` passes locally before the push; before this change it failed on the three paths.
- `git diff 76ee339b -- src/change.rs src/change/sdd_version.rs tests/integration/change.rs` is empty, so the reviewed bytes are unchanged.
- CI on the archived tip: Lifecycle gate, trust and the required gates pass.
- The integration test already in `tests/integration/change.rs`, `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`, passed in the verify lane on `47ac3bb0` (2833 unit + 495 integration).
