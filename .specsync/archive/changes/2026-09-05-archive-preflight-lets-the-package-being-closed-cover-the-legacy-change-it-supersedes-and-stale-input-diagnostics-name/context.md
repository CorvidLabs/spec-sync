---
change: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
artifact: context
---

# Context

Hit on CorvidLabs/swift-algorand `chore/specsync-6` (v6.0.0-rc.12, ac796b8): a workflow-v2 change superseding six `Sources/` inputs of legacy `CHG-0001` was rated `exact` by `change audit`, and `change finalize` then refused it: "archive post-move preflight would invalidate `CHG-0001-…`: delivery input `Sources/Algorand/ApplicationTransaction.swift` (owner `algorand`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen CHG-0001-…`". `ship-status` said product tip done, review tip done, archive tip pending — on every attempt.

Two faults, both in the successor walk of `validate_accepted_inputs_recursive`:

1. The walk authenticated the package being closed with no `PendingArchiveClose` token, and checked its succession tuple through `semantic_tuple_transition_is_valid`, which hands the anchor from `authenticated_accepted_transition` to `git merge-base`. For a package whose acceptance is not in history that anchor is the label `working-tree-closing-evidence`, not a commit, so the tuple read as "does not hold" and the only successor able to cover the legacy change was dropped — during the very finalize that was covering it. `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` reproduces the field message on ac796b8.
2. Every refusal in the walk was `Err(_) => continue`, so the diagnostic said no successor existed (#685's family) and recommended `reopen` of the legacy change — which replays its canonical delta over the successor's materialization: its `## ADDED` section block conflicts with the successor's `## MODIFIED` of the same section, and its `## MODIFIED` requirement blocks overwrite the amended wording.

3. Found on the second field round, with the fixed binary: `finalize` succeeded, and `change audit` then still reported `CHG-0001` as uncovered — with the OLD wording, so no successor had been refused; none had been seen. `audit_project` takes the active-only path of `check_project_with_command_output`, which built one map from `list_changes_checked` and handed it to `terminal_evidence_results_with_records` as both the records to evaluate and the successor universe. A finalized successor is an archive, so it was never a candidate. Evaluating fewer records must never mean offering fewer successors.

Ruled out: using `verification.commit` as the tuple anchor for the closing package. `validate_verification_for_commit_binding` states that the commit is an informational correlation key and does not bind the tree the manifest signed; the working tree is what the closing evidence speaks for, and `accept_change_with_gate` already checked the base ancestry against HEAD when it signed the tuples.

Not fixed here, follow-up: two stacked workflow-v2 changes on one branch that both touch a legacy predecessor's inputs cannot finalize in either order. A reviewed-but-unfinalized v2 change has no acceptance manifest — v2 has no resting `accepted` state (`change accept` refuses v2; finalize accepts and archives in one process) — so it cannot cover anything, and "finalize one at a time" means the other change is never terminal while the first runs its preflight. That is a lifecycle design question, not a defect of this walk.
