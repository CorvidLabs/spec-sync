---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: plan
---

# Plan

## Approach

Move the approval's checks into one ordered list that both the approval and its dry run walk, add the dry
run as a second walk that collects each step's first problem and writes nothing, and expose it as
`specsync change approve <id> --dry-run`. The approval's own behaviour is pinned by its existing tests and
by a table test that runs the real approve after each dry run.

## Out of scope

- Listing every problem inside one check. Leif chose the first problem from each check for this release.
- Any change to what a validator accepts or to a refusal's text.
- Chough's adoption of the flag (chough#404) and its `change ready` (chough#311, #406).
- A dry run for `change review`, `check` or `finalize`.

## Steps

1. Failing tests: the integration table of refused drafts and the clean draft, the unchanged-tree
   comparison, the two-problem draft, the missing id, the sent-back change, the relay flags; the argument
   parsing test.
2. `src/change/approval_checks.rs`: the step list and `ApprovalRun`; split building the approval record
   from writing it; make `approve_definition_with_relay` walk the list.
3. `approve_definition_dry_run` and its report types.
4. `--dry-run` on `change approve` in `src/cli.rs`, and the text and JSON output with the exit code in
   `src/commands/change.rs`.
5. Public API rows and `files:` entries in the three specs; `change check` to add the requirements.
6. Docs: `site/src/content/docs/cli.md`, `CHANGELOG.md`, the `AGENTS.md` command table, and a lesson
   plane in `specs/change/change.3md`.
7. `fledge lanes run verify`, then the pre-push gate.

## Risks

- The approval refuses something it accepted before, or says it differently. Held by walking the list in
  today's order, by the existing approve tests, and by the table test's real approve after each dry run.
- A step that writes slips into the dry run. Held by comparing every file of the project before and after,
  with and without the lock file and the hash cache.
- Two entries for one cause, when a later step reads what an earlier one refused. Accepted and stated in
  the definition: only three skip rules.
- PR #1093 edits the end of `cmd_change` for its own exit code. The two meet in a few lines; whichever
  merges second rebases.

## Constraints consulted

- CHANGE-6 and CHANGE-6.a: one approval, bound to its content. The dry run records nothing and is never an
  approval.
- REQ-change-133: the refusal texts for an unfinished definition stay as they are.
- `specs/change/change.3md`: `check_project` and `audit_project` share one private implementation,
  because a rule kept in two places drifts.
- `specs/change/change.3md`: a printed recovery command is a promise that it runs as printed, so the dry
  run reports the approval's messages unedited.
- `specs/change/change.3md` (doctor): a dry run reuses the lifecycle lock's no-symlink walk so it predicts
  that refusal.
- Leif on #1091, 2026-11-18: "First problem from each check".
