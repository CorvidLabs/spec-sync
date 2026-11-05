# Lesson bundle: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse a stale modified delta instead of overwriting newer canonical text
- **Kind**: BugFix
- **Specs**: change
- **Paths**: INTENT.md, hi/change.md, specs/change, site/src/content/docs/deltas.md, src/change/collect_requirement_ids.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/stale_modified_delta.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs
- **Acceptance**: A change approved against older contract text never silently overwrites newer text another change put there.
- **Acceptance**: Checking that change stops and names the requirement or section whose text moved.
- **Acceptance**: The project health check (audit) reports the same problem before anything is lost.
- **Acceptance**: The message tells me the way forward: rewrite the change onto the current text and approve it again.

## Evidence

- Verification commit: `ec6d9e2f27ef27fd90ff068f391b5b0021251e75`
- Base commit: `f82eb09e2d3da852ff58e0cfbb734ab6faba724e`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #843. While merging main into CorvidLabs/chough PR #77, `specsync change check` applied #77's approved REQ-cli-007 MODIFIED body over the version #71 had just landed on main, deleting #71's paragraph and acceptance criterion. It exited 0, and `change audit` passed. It was caught only by grepping before and after.

Leif confirmed the criteria (CHANGE-11 to CHANGE-11.c) in a Grok Bot widget on 2026-11-04 at 7:34 AM MST. He chose to "Protect only changes approved from now on", so approvals written before this field are not re-judged.

## From the change's design.md

# Design

- New serde-optional `ApprovalRecord.approved_delta_bases`. It maps a readable key such as `cli: REQUIREMENT REQ-cli-007` to a framed digest (domain `specsync.approved-delta-base.v1`) of the canonical block at approval: line endings folded, surrounding blank lines trimmed, absence recorded as absence.
- Writers:
  - `append_approval` (definition gate only) and the `--portable-5-0-1` pair read the bases beside the bodies.
  - The normalizing approval in `accept_change` carries the superseded approval's bases forward rather than re-reading an already-applied tree.
- `ensure_approved_delta_bases_current` runs in `materialize_change_deltas` directly after the body guard, above the `canonical_applied` short-circuit and before any write. Audit runs it only once the body guard holds.
- An item that is already applied is converged and never refused. A missing key, or a ledger without bases, is unknown, not violated (same reading as REQ-change-089/090).

## From the change's testing.md

# Testing

`a_stale_modified_delta_never_overwrites_text_main_added_after_materialization` and `a_stale_modified_delta_never_overwrites_text_main_added_before_the_first_check` fail on main f82eb09, which overwrites main's text and returns Ok. They pass with the fix. Run with `cargo test --bin specsync stale_modified`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-122 | `a_stale_modified_delta_never_overwrites_text_main_added_after_materialization`, `a_stale_modified_delta_never_overwrites_text_main_added_before_the_first_check`, `audit_reports_a_stale_modified_delta`, `a_rebased_and_re_approved_delta_materializes_over_the_moved_requirement`, `main_moving_an_unrelated_block_does_not_refuse_the_check`, `an_approval_without_recorded_bases_keeps_materializing` |

## Where these lessons go

- `specs/change/change.3md`
