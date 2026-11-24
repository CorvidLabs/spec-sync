# Lesson bundle: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The cutoff archive-root probe reads the archive root as a literal path
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs
- **Acceptance**: for a project in a directory named :(exclude)app whose trusted cutoff has no archive root, the cutoff-tree check answers from the cutoff's own tree instead of reporting that the archive tree could not be read; an unreadable cutoff archive tree is still an error

## Evidence

- Verification commit: `6eab8fb4b71503348a3d1113d9ed50a791714907`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#990, a Codex P2 on #988 deferred by Leif's call (P2 policy: issue, reply, resolve).
`workflow_v1_record_present_in_cutoff_tree` (`src/change/validate_loaded_change.rs`) first reads
`<cutoff>:<archive root>`. When that fails it probes `ls-tree --full-tree -z --name-only <cutoff>
-- <archive root>` to tell "the cutoff has no archive root" (absent) from "the archive tree is
there but unreadable" (an error, REQ-change-145). The probe passed the repository-relative path
as a plain pathspec. For a project in a directory whose name begins with pathspec magic, such as
`:(exclude)app`, `ls-tree` rejects the magic and fails, so a cutoff with no archive root was
reported as "could not be read". That fails closed, but it is the wrong verdict.

The fix passes `:(top,literal)<archive root>`, as the module's other history queries do (#851),
and keeps comparing listed entries with the plain path. The unreadable-tree error path is
unchanged (`an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change`).

The old `0xleif/fix/g-literal-929-930-931-990` WIP branch (2026-11-06) is stale; this is a fresh
change from main for #990 alone.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-145 | `change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory` (red before the fix, green after) and the existing `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` |
| REQ-change-136 | `cutoff_tree_verdict_equals_the_history_walk_verdict` still passes |

## How it runs

`change check` runs only the in-process spec-code sync. The unit tests run locally with
`cargo test --bin specsync cutoff`, and in hosted CI's full suite at the archived head.

## Where these lessons go

- `specs/change/change.3md`
