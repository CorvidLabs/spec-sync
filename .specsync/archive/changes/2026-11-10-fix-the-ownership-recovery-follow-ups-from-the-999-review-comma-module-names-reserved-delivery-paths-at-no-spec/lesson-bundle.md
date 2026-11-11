# Lesson bundle: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fix the ownership recovery follow-ups from the #999 review: comma module names, reserved delivery paths at no-spec approval, listing-bound silence, constraints renewal, registry specs, and directory scopes
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/historical_definition_digest_matches.rs, src/change_tests/approve_ownership_names_the_spec.rs
- **Acceptance**: A no-spec-change draft listing a reserved delivery path approves; a comma-bearing module name never produces a non-round-tripping printed answer; an over-bound spec listing never silently omits an owner from recovery; ownership recovery re-prompts constraints when it clears them; a registry canonical spec outside specs_dir joins owner discovery; a directory scope with a source-like extension is not ownership-checked as a file

## Evidence

- Verification commit: `b0563e3f96c644539022bbaf91a17897f69b0649`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

## Decisions

- The reserved-delivery exemption lives in
  `production_source_lacks_canonical_owner_with_evidence`, so approve-time validation and
  `correct-owner` discovery share it, matching the finalize-side exemption
  (`path_is_recognized_delivery_metadata` in the manifest builder).
- For comma-bearing module names the printed answer is omitted rather than escaped:
  `change answer` splits on commas and there is no escaping layer to fix, so the honest
  recovery is prose naming the modules plus the `files:` edit.
- The listing bound fails closed only where the index informs recovery: the ownership gate
  itself still works from declared specs and configured `owns`, which need no listing.

## Ruled out

- Changing `change answer` splitting or adding escaping: a wire-format change for a
  diagnostic's sake.
- Raising or removing the 10,000-entry bound: the bound protects every caller; the fix is
  to stop presenting truncated results as complete.

## From the change's testing.md

# Testing

- Reserved delivery path: a no-spec-change draft over `.github/scripts/check.py` with
  `source_dirs = ["."]` approves (previously refused).
- Comma module: a module named `billing,legacy` yields recovery prose, never a printed
  `change answer` command that would mis-split.
- Listing bound: a fixture with more than 10,000 spec-tree entries makes the recovery say
  the listing is incomplete instead of omitting the owner.
- Constraints renewal: the printed recovery names the constraints re-answer when answering
  `affected_specs` clears it.
- Registry: a registry-registered canonical spec outside `specs_dir` appears in owner
  discovery and the printed declaration answer.
- Directory scope: a no-spec-change draft scoped to `src/generated.rs/` holding only a
  README approves and is not ownership-checked as a file.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-053` | the six new tests above plus the existing ownership-gate tests in `src/change_tests/substantive_plan.rs` and `src/commands/change.rs`, unchanged and passing |

## Where these lessons go

- `specs/change/change.3md`
