# Lesson bundle: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The workflow-v2 baseline history probe reads the baseline path literally
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs
- **Acceptance**: in a project directory named :(exclude)app with no committed baseline, the reachable-history probe answers false and reading the baseline gives none instead of reporting it deleted

## Evidence

- Verification commit: `a016fc772ce37bceb7befa72378c503cb6070845`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#1074, found while writing #990's test. `workflow_v2_baseline_exists_in_reachable_history`
(`src/change/validate_loaded_change.rs`) ran `rev-list --full-history --max-count=1 HEAD -- <path>`
with the repository-relative baseline path as a plain pathspec. For a project at `:(exclude)app/`
Git read it as an exclude that matched every other commit, so a never-committed baseline was
reported as "committed workflow-v2 baseline was deleted". The fix passes
`:(top,literal)<baseline path>`, as #851 and #990 did for the module's other history queries.
Leif approved it as a bug fix on 2026-11-13 ("#1074 + #1075 (new)").

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).

## From the change's testing.md

# Testing

| Behaviour | Evidence |
|---|---|
| A never-committed baseline is absent in a pathspec-magic project directory | `change::tests::a_never_committed_baseline_is_not_read_as_deleted_in_a_pathspec_magic_project_directory` (red before the fix, green after) |
| Deleted committed baselines are still refused | the existing workflow-v2 baseline tests |

Locally: `cargo test --bin specsync baseline`; hosted CI runs the full suite at the archived head.

## Where these lessons go

- `specs/change/change.3md`
