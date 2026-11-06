---
change: cover-the-two-paths-900-changed-outside-its-declared-scope
artifact: testing
---

# Testing

The covered code already has tests from #900:

| Criterion | Test |
| --- | --- |
| CHANGE-6.d | `correction_digests_leave_embedded_relays_out` in `src/change_tests/relayed_decision.rs`: both correction digests, including `correction_prefix_digest`, equal the digests computed with the relay removed. The test fails with the projection removed. |
| CHANGE-13.b | `status_and_show_json_report_approved_delta_requirements` in `tests/integration/change.rs`: approved status JSON carries `recorded_claims` with the approval claim, the key set is otherwise exact, and a draft omits it. |

The local verify lane passed at `39d3d54d` (2838 unit, 505 integration). Before review it runs again with `change audit --strict`, and the audit must pass on the archive tip before push.
