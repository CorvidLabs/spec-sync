# Lesson bundle: commit-identities-from-change-records-are-used-only-as-full-commit-ids

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Commit identities from change records are used only as full commit ids
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/retirement.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/commit_identity_guards.rs, src/commands/change.rs, tests/integration/change.rs
- **Acceptance**: A change's base commit, verification and scoped-review commits, and bootstrap and reopening anchors read from records reach Git as revisions only when they are full commit ids, after --end-of-options; a recorded base that is not one makes the coverage gate fall back to the no-record base instead of failing

## Evidence

- Verification commit: `8d49973e96de0830bf07d4523ce161194c9f49f7`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

The same class of input handling was hardened for `files_changed_since_base` in PR #1037's review.
A read-only sweep of the other Git calls that take values from change records found the sites this
change covers. REQ-change-013 already requires persisted workspace values to be validated before
use, so this is a bug fix with no requirement text.

## From the change's design.md

# Design

Commit identities that SpecSync reads from change records are persisted workspace values, so they
are validated before use (REQ-change-013). Each place that passes one to Git as a revision now
requires a full commit id (`is_canonical_commit_id`), and the revision follows `--end-of-options`,
as `specsync diff` already does.

- The coverage diff base (`recorded_diff_base`): a recorded base that is not a commit id falls back
  to the base used when no change records one (`HEAD~1...HEAD`, or `HEAD`).
- `--end-of-options` goes before the base in both coverage diffs, `comparison_base_commit`'s
  merge-base, and the policy read at the comparison base.
- A full commit id is required for the semantic-succession base, the succession-base worktree (which
  also takes `--end-of-options`), shallow-history completeness, bootstrap records, verification and
  scoped-review commits, the reopening anchor, and status evidence.
- `--` goes before the policy path in the versioned-SDD probe.

A value that is not a commit id is treated as absent (or as not an ancestor), never passed on.

## From the change's testing.md

# Testing

- Unit `a_recorded_commit_that_is_not_a_full_commit_id_is_never_handed_to_git` (fails on main).
- Integration `coverage_uses_a_changes_recorded_base_only_when_it_is_a_full_commit_id` (fails on main
  with "unable to inspect changed paths").
- The targeted succession, shallow-history, retirement, scoped-review, status and ship-status tests
  still pass.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
