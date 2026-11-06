---
change: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory
artifact: context
---

# Context

Audit D2 (spec-sync#879): in a project rooted at `app/` below its Git repository root, `change check --commit` exits 1 with "Git returned an out-of-scope modified path `app/specs/greet/greet.spec.md`". The path is the change's own materialized spec. The repository-root control completes.

Reproduced on main 884a33d3 with a fresh minimal project in `app/` (one change on `src/auth.rs`, spec `auth`): "Git returned an out-of-scope modified path `app/specs/auth/auth.spec.md`".

Cause: working-tree evidence capture (`src/change/consider_accepted_evidence_anchor.rs`) passes project candidates to Git as `:(top,literal)<repo prefix><path>` and checks each returned path with `candidate_scope_admits`. `git diff-files --name-only` reports paths relative to the repository root, so every path carries the `app/` prefix and none matches a project-relative candidate. `ls-files` in the same function reports paths relative to the working directory, which is the project root, and is not affected.

Related: #851 / PR #868 fixed the history pathspecs for subdirectory projects; this is the working-tree side.
