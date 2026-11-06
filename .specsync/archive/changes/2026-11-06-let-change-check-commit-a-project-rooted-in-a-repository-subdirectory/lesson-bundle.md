# Lesson bundle: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let change check commit a project rooted in a repository subdirectory
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, tests/integration.rs, tests/integration/subdirectory_check_commit.rs, CHANGELOG.md
- **Acceptance**: In a project rooted in a subdirectory of its Git repository, `change check <id> --commit` on a change whose source and materialized spec live in the project completes: it commits the materialized spec and the verification evidence, as it does at the repository root.
- **Acceptance**: Working-tree evidence capture reads Git's modified-path output relative to the project, so the change's own spec (`app/specs/<module>/<module>.spec.md` from Git, `specs/<module>/<module>.spec.md` in the project) is in scope.
- **Acceptance**: A path Git reports outside the project or outside the change's candidates is still refused with the same out-of-scope error naming that path.
- **Acceptance**: A project at the repository root behaves exactly as before. No gate is loosened.
- **Acceptance**: The auditor's subdirectory steps are an integration test that fails on main first.

## Evidence

- Verification commit: `8b0be3cbedc656244e3b9a9917eff8423f4154fc`
- Base commit: `884a33d3c0cb95680dae786ff25688569ee16c96`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Audit D2 (spec-sync#879): in a project rooted at `app/` below its Git repository root, `change check --commit` exits 1 with "Git returned an out-of-scope modified path `app/specs/greet/greet.spec.md`". The path is the change's own materialized spec. The repository-root control completes.

Reproduced on main 884a33d3 with a fresh minimal project in `app/` (one change on `src/auth.rs`, spec `auth`): "Git returned an out-of-scope modified path `app/specs/auth/auth.spec.md`".

Cause: working-tree evidence capture (`src/change/consider_accepted_evidence_anchor.rs`) passes project candidates to Git as `:(top,literal)<repo prefix><path>` and checks each returned path with `candidate_scope_admits`. `git diff-files --name-only` reports paths relative to the repository root, so every path carries the `app/` prefix and none matches a project-relative candidate. `ls-files` in the same function reports paths relative to the working directory, which is the project root, and is not affected.

Related: #851 / PR #868 fixed the history pathspecs for subdirectory projects; this is the working-tree side.

## From the change's testing.md

# Testing

New file `tests/integration/subdirectory_check_commit.rs`: a Git repository whose SpecSync project is `app/`, one change approved for `src/auth.rs` with an `auth` delta, the implementation committed, then `change check --commit`. It must exit 0, commit the materialized `app/specs/auth/auth.spec.md`, and leave the change `verifying`. On main 884a33d3 it fails with the out-of-scope error.

A unit test on the path normalization covers the refusals: a path without the repository prefix, and a prefixed path that is not a candidate.

The repository-root control: the existing `change check --commit` integration tests stay green.

## Results

- `check_commit_completes_for_a_project_rooted_in_a_repository_subdirectory` failed on main 884a33d3 with "Git returned an out-of-scope modified path `app/specs/auth/auth.spec.md`" and passes with the change.
- `a_subdirectory_project_reads_diff_paths_relative_to_itself` and `a_path_outside_the_project_is_still_out_of_scope` cover the normalization and the refusals.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-128 | `check_commit_completes_for_a_project_rooted_in_a_repository_subdirectory`, `a_subdirectory_project_reads_diff_paths_relative_to_itself`, `a_path_outside_the_project_is_still_out_of_scope` |

## Where these lessons go

- `specs/change/change.3md`
