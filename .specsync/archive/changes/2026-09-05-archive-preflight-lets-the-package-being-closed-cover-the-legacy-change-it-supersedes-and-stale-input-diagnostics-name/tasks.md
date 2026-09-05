---
change: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
artifact: tasks
---

# Tasks

- [x] Forward the archive preflight's `PendingArchiveClose` token through `validate_accepted_inputs_recursive`, `later_sequence_owner_covers_historical_input`, `authenticate_accepted_evidence[_with_anchor]`, and `semantic_tuple_transition_is_valid`; every reader passes `None`.
- [x] Name the `working-tree-closing-evidence` anchor as a constant and, when a successor's transition is that label, check its succession tuple against the working tree with base ancestry against HEAD (`acceptance_entry_digest_in_tree`, shared with the detached-worktree path).
- [x] Record every refusal in the successor walk as `RejectedSuccessor { workflow_version, reason }` and render it from `stale_input_remediation_reason`; pre-filter candidates with `declares_succession_obligation` so each recorded refusal is about a successor that claimed the input.
- [x] Do not offer `change reopen` of a workflow-v1 change beside a refused workflow-v2 successor; direct to finishing the successor.
- [x] Split `terminal_evidence_results_with_records` into the records it evaluates and the successors it offers; the active-only audit loads archived records as candidates only when an active terminal record exists.
- [x] Regression tests, REQ-change-020/024/036, REQ-change-audit-project-001 and the `Error Cases` delta, and the lesson in `specs/change/context.md`.
