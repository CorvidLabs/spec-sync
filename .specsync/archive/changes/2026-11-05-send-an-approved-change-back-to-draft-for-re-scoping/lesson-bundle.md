# Lesson bundle: send-an-approved-change-back-to-draft-for-re-scoping

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Send an approved change back to draft for re-scoping
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args
- **Paths**: INTENT.md, hi/change.md, specs/change, specs/cmd_change, specs/cli_args, site/src/content/docs/cli.md, site/src/content/docs/workflow.md, src/change.rs, src/change/send_back.rs, src/change/sdd_version.rs, src/change/consider_accepted_evidence_anchor.rs, src/change_tests.rs, src/change_tests/send_back_for_rescope.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs, src/cli.rs, src/commands/change.rs
- **Acceptance**: When a change's scope turns out wrong after approval, I can send it back to be re-scoped instead of deleting it by hand.
- **Acceptance**: Sending it back needs a named person and a reason, and both stay in the change's history.
- **Acceptance**: Sending it back cancels its approval, so the new scope needs a fresh one.
- **Acceptance**: Nothing already recorded about the change is lost.

## Evidence

- Verification commit: `fb516245bedfcecc64302daf1ba203d274c79e63`
- Base commit: `f82eb09e2d3da852ff58e0cfbb734ab6faba724e`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

Issue #844. After a spec split in CorvidLabs/chough (#71), the verifying change of chough#69 needed a different scope. `change answer` only edits drafts, `change correct` only fixes accepted metadata, and `change reopen` only reopens accepted or archived work, so the only way out was deleting the workspace by hand. That loses its audit history, and while it stays open it blocks Archive.

Leif chose "Both A and B" in a Grok Bot widget on 2026-11-04 at 7:34 AM MST. That confirms option A (send back for re-scoping, CHANGE-12 to CHANGE-12.c), which this change implements. Option B (withdraw plus supersede) is a separate change and is not in this scope.

## From the change's design.md

# Design

- `SendBackRecord` is appended to `ApprovalLedger.send_backs` (serde-optional, omitted when empty, so existing ledgers keep their bytes). It holds actor, reason, timestamp, `from_state`, `to_state` (draft), `cancelled_approvals` (the ledger's approval count at send-back) and whether canonical text had been applied, the items it had applied (`applied_items`) and the modules whose Change Log carried its row (`applied_modules`).
- `send_back_change` takes the project lock and writes `approvals.json`, `state.json` and `change.md` together. It sets the state to draft and `canonical_applied` to false.
- `effective_definition_approval` refuses when the latest definition approval's index is below the latest send-back's `cancelled_approvals`. Every gate that reads the definition approval therefore sees the cancellation, and nothing is deleted.
- Spec text the change already applied stays in place. The send-back lists it as `applied_items` (module, operation, target, key, body, and the block's pre-change text from the parent of the first commit carrying the change's Change Log row, or HEAD before that row is committed) for every module whose Change Log names the change; REMOVED items are listed while their block is still gone. `prepare_module_application` converges whenever the ledger holds a send-back, not only while `canonical_applied` is set, so a kept item already in the tree is skipped and the version bump and Change Log row are never repeated. A kept ADDED item whose block still carries the body a send-back listed is applied as a replacement.
- `ensure_no_orphaned_applied_text` runs in verify (so in `change check`) and in accept. It refuses while an item listed by a send-back is no longer declared by the delta (same module, target and key) and the canonical tree still carries what the change did (its body, or a removed block's absence), naming the spec file and the item and the text to restore. It also refuses a module dropped from `affected_specs` whose Change Log still carries the change's row.
- `ensure_committed_send_backs_kept` reads the ledger history with one cached `git log -G send_backs` per ledger path and requires the change's send-backs in every listed revision to be a prefix of the current ones. The project audit runs it for every change it audits, active or archived.
- `resolve_definition_approval_event` (historical and archived evidence), the scoped-review approver lookup and the scope-expansion summary skip approvals the send-backs cancelled.
- `ensure_committed_send_backs_kept` runs with the definition gate: when `approvals.json` is committed at HEAD, its send-backs must be an unchanged prefix of the current ones, so deleting or editing a committed send-back cannot bring the cancelled approval back.
- CLI: a `ChangeAction::SendBack { id, actor, reason }` command that renders text or JSON.

## From the change's testing.md

# Testing

The first five tests in `src/change_tests/send_back_for_rescope.rs` were written before the implementation and all failed against a stub `send_back_change`. The finalize-and-audit test and the CLI grammar test came afterwards. All seven pass with the implementation.

Review round on e52f77d (SpecSync bot, REQUEST_CHANGES). Three tests were added and the finalize test was rewritten to use a change that owns a spec, before the fix; the failing-first run is in `/workspace/pr-burndown/ss846-review-failing-first.log`. Against e52f77d, `a_re_scope_that_drops_applied_text_is_refused_until_the_text_is_removed`, `the_definition_approval_event_resolver_honours_a_send_back`, `deleting_a_committed_send_back_does_not_revive_the_cancelled_approval` and the rewritten `a_sent_back_change_finalizes_and_its_archive_passes_audit` all failed. All ten pass with the fix. Run them with `cargo test --bin specsync -- send_back sent_back sending_back only_an_approved_unfinished re_scope`.

The command output was inspected on a scratch project with the built binary: text output, `--json` output carrying `change` and `audit`, and the non-zero refusal for a draft.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-123 | `a_verifying_change_sent_back_can_be_re_scoped_and_delivered`, `sending_back_needs_a_named_person_and_a_reason`, `a_send_back_is_kept_in_the_change_history_and_loses_nothing`, `sending_back_cancels_the_approval_until_a_fresh_one`, `only_an_approved_unfinished_change_can_be_sent_back`, `a_sent_back_change_finalizes_and_its_archive_passes_audit`, `a_re_scope_that_drops_applied_text_is_refused_until_the_text_is_removed`, `the_definition_approval_event_resolver_honours_a_send_back`, `deleting_a_committed_send_back_does_not_revive_the_cancelled_approval`, `a_sent_back_change_with_a_removed_item_can_be_checked_again`, `a_sent_back_removed_only_change_materializes_again`, `a_sent_back_change_can_change_the_body_of_an_item_it_added`, `a_dropped_modified_item_is_refused_with_advice_to_restore_its_earlier_text`, `a_dropped_removed_item_is_refused_until_the_section_is_restored`, `a_module_dropped_from_the_scope_is_refused_while_it_keeps_the_change_log_row`, `a_send_back_deleted_in_a_later_commit_is_refused_and_audited` |
| REQ-cmd-change-026 | Scratch-project CLI inspection of text, `--json` and refusal output |
| REQ-cli-args-021 | `change_send_back_requires_and_collects_audit_inputs` |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
