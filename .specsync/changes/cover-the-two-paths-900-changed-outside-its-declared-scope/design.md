---
change: cover-the-two-paths-900-changed-outside-its-declared-scope
artifact: design
---

# Design

No code or spec text changes. The change declares the two paths so that the pull request's audit covers them:

- `src/change/collect_requirement_ids.rs`: `correction_prefix_digest` serializes `correction_digest_projection(correction)`, which clears `relayed_by` on the embedded approvals, the same projection `corrected_view_digest` uses.
- `tests/integration/change.rs`: the status JSON key set is exact. After approval it adds `recorded_claims`, which equals `[{"gate": "definition", "name": "Reviewer"}]`, and a draft omits it.
