# Lesson bundle — archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Archive preflight lets the package being closed cover the legacy change it supersedes, and stale-input diagnostics name the refused successor
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: `specsync change finalize` of a workflow-v2 change that supersedes inputs of a legacy accepted change succeeds: the archive post-move preflight authenticates the package being closed with its closing token and reads its succession entries from the working tree, so the legacy change is successor-covered before and after the archive commit
- **Acceptance**: A stale delivery-input diagnostic names every successor that claimed the input and was refused, with the reason (sorted by successor ID), instead of reporting that no successor covers it
- **Acceptance**: When the stale predecessor is workflow v1 and a refused claiming successor is workflow v2, the diagnostic directs the operator to finish that successor and does not offer `specsync change reopen` of the legacy change
- **Acceptance**: Regression tests: finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change and stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication fail on ac796b8 and pass with the fix

## Evidence

- Verification commit: `3ec50edbb84382e43a3be9b422bfba46ad8dcb89`
- Base commit: `6b1717038edb467d95bb483861f0c076da76deb5`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Hit on CorvidLabs/swift-algorand `chore/specsync-6` (v6.0.0-rc.12, ac796b8): a workflow-v2 change superseding six `Sources/` inputs of legacy `CHG-0001` was rated `exact` by `change audit`, and `change finalize` then refused it: "archive post-move preflight would invalidate `CHG-0001-…`: delivery input `Sources/Algorand/ApplicationTransaction.swift` (owner `algorand`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen CHG-0001-…`". `ship-status` said product tip done, review tip done, archive tip pending — on every attempt.

Two faults, both in the successor walk of `validate_accepted_inputs_recursive`:

1. The walk authenticated the package being closed with no `PendingArchiveClose` token, and checked its succession tuple through `semantic_tuple_transition_is_valid`, which hands the anchor from `authenticated_accepted_transition` to `git merge-base`. For a package whose acceptance is not in history that anchor is the label `working-tree-closing-evidence`, not a commit, so the tuple read as "does not hold" and the only successor able to cover the legacy change was dropped — during the very finalize that was covering it. `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` reproduces the field message on ac796b8.
2. Every refusal in the walk was `Err(_) => continue`, so the diagnostic said no successor existed (#685's family) and recommended `reopen` of the legacy change — which replays its canonical delta over the successor's materialization: its `## ADDED` section block conflicts with the successor's `## MODIFIED` of the same section, and its `## MODIFIED` requirement blocks overwrite the amended wording.

3. Found on the second field round, with the fixed binary: `finalize` succeeded, and `change audit` then still reported `CHG-0001` as uncovered — with the OLD wording, so no successor had been refused; none had been seen. `audit_project` takes the active-only path of `check_project_with_command_output`, which built one map from `list_changes_checked` and handed it to `terminal_evidence_results_with_records` as both the records to evaluate and the successor universe. A finalized successor is an archive, so it was never a candidate. Evaluating fewer records must never mean offering fewer successors.

Ruled out: using `verification.commit` as the tuple anchor for the closing package. `validate_verification_for_commit_binding` states that the commit is an informational correlation key and does not bind the tree the manifest signed; the working tree is what the closing evidence speaks for, and `accept_change_with_gate` already checked the base ancestry against HEAD when it signed the tuples.

Not fixed here, follow-up: two stacked workflow-v2 changes on one branch that both touch a legacy predecessor's inputs cannot finalize in either order. A reviewed-but-unfinalized v2 change has no acceptance manifest — v2 has no resting `accepted` state (`change accept` refuses v2; finalize accepts and archives in one process) — so it cannot cover anything, and "finalize one at a time" means the other change is never terminal while the first runs its preflight. That is a lifecycle design question, not a defect of this walk.

## From the change's design.md

# Design

- One token, forwarded, never minted: the successor walk gains `pending: Option<&PendingArchiveClose>`; every reader passes `None` (documented on `PendingArchiveClose`), and the archive preflight passes `pending_close.as_ref()`. `is_closing` keeps the token inert for every package except the one being closed, and a post-move resume still gets none.
- The working-tree anchor is decided by the label `authenticated_accepted_transition_for` already returns (`WORKING_TREE_CLOSING_EVIDENCE_ANCHOR`), not by the token, because the same shape exists for every reader between `finalize` and the archive commit. Once the archive commit exists, history is again the sole anchor. The successor entry is read by `acceptance_entry_digest_in_tree`, the same code the detached-worktree path uses.
- Refusals become data: `RejectedSuccessor { workflow_version, reason }` in a `BTreeMap` keyed by successor ID (deterministic order), rendered as "successor `<id>` was rejected: <reason>". The three formerly OR'd checks are split into distinct reasons, and the two `Result<bool>` predicates report a decided negative and a failure to evaluate differently (#743).
- Candidates are pre-filtered by the declared obligation (`declares_succession_obligation`, now shared with `legacy_semantic_successor_tuple`). This is equivalent to the prior behaviour — authenticated tuples are one-to-one with declared obligations, and the legacy reconstruction required the declaration — and it keeps every recorded refusal about a successor that actually claimed the input.
- Remediation depends on the two workflow versions: a v1 predecessor with a refused v2 claiming successor is steered to `specsync change finalize` via `change status <successor>` and is never offered `change reopen <predecessor>`; every other combination keeps the established verify-and-accept-or-reopen wording.
- Evaluated records and successor candidates are two parameters of `terminal_evidence_results_with_records`, not one. The active-only audit keeps evaluating active terminal records alone and keeps its empty common case free of archive reads; when an active terminal record exists it loads `list_all_changes_checked` as the candidate universe. Nothing archived is evaluated there, and the declared-obligation pre-filter means only an archive that claims the input is ever authenticated.

## From the change's testing.md

# Testing

- `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` — DISCRIMINATOR for the finalize defect: a legacy (v1) predecessor is accepted and committed, a v2 successor supersedes every `auth`-owned input of it, then approve → check → commit → check → review → finalize. Fails on ac796b8 with the field message; passes with the fix. Its tail proves the predecessor is successor-covered before and after the archive commit on BOTH `check_project` and `audit_project` (the audit's `terminal_evidence` rates the predecessor `SuccessorCovered`; on 507c14e4 the audit reported it stale with the "no successor" wording because its candidate universe was active-only), and that disturbing the successor's inputs afterwards yields the refused-successor diagnostic — identical on both surfaces — that steers to `change status <successor>` and away from `change reopen <predecessor>`.
- `stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication` — DISCRIMINATOR for the silent refusal: a covering v1 successor whose `verification.json` is flipped to `passed: false` is named with "its accepted evidence did not authenticate: accepted change has failed verification evidence". On ac796b8 the message claims no successor covers the input.
- `stale_accepted_change_error_names_covering_successor_with_stale_evidence` — CONTROL, re-anchored on the shared `accepted_change_with_covering_successor` fixture: the stale-successor case now carries the nested reason and keeps the verify-and-accept-or-reopen remediation for a v1/v1 pair.
- `stale_accepted_change_error_names_uncovered_input_and_reopen_remediation` and `stale_accepted_change_error_names_exact_only_input_and_audited_reopen` — CONTROLS, unchanged: the no-claiming-successor and exact-only messages are byte-identical.
- `fledge run lint` (clippy, `-D warnings`), `fledge lanes run pre-push`, `fledge lanes run verify`.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-020 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |
| REQ-change-024 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |
| REQ-change-audit-project-001 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` (audit assertions) |
| REQ-change-036 | `stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication`, `stale_accepted_change_error_names_covering_successor_with_stale_evidence`, `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |

## Where these lessons go

- `specs/change/context.md`
