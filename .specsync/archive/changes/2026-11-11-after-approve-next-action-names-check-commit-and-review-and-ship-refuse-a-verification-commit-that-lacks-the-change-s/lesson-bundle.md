# Lesson bundle: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: After approve, next_action names check --commit, and review and ship refuse a verification commit that lacks the change's work
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: specs/change/requirements.md, specs/change/change.spec.md, specs/change/testing.md, src/change/historical_definition_digest_matches.rs, src/change/sdd_version.rs, src/commands/change.rs, tests/integration/change.rs, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/change_tests/verification_ignored_paths.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs
- **Acceptance**: After approve, status and show name specsync change check <id> --commit. change review and change ship refuse when the verification commit does not contain the change's work, and do not bind a review to a commit that lacks that work.

## Evidence

- Verification commit: `51f39a18c7edf3163a219541a2af4c760bf4d6d6`
- Base commit: `ad8544e9da79725f499b38ae4d4b3a052be5f807`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #535, as re-measured on 2026-08-14 and again on current main
(`ad8544e9`, specsync 6.2.0, 2026-11-11 MST).

The filing made two claims. The second — that the resulting archive cannot be
committed, with every recovery verb refusing — does not hold on 6.0.0 or on
6.2.0: `git add` and `git commit` succeed, and there is no hook refusal. That
half is out of scope here.

The first claim still holds. After `change approve`, `change status` and
`change show` name `specsync change check <id>` with no `--commit`. Following
that command, then `change review` and `change ship`, all succeed against a
tree where the change's work is uncommitted. Review binds to the seed commit,
which does not contain the work. On 6.2.0, `change ship` now writes the
archive commit itself and includes the uncommitted work in that commit, while
`verification.json` and `review.json` still name the seed. `ship-status` then
reports the review stale. The guided path is still not sufficient, and
review/ship still accept a verification commit that lacks the work.

After a bare `change check`, text `next_action` already says
`change check <id> --commit` if needed. The gap is after approve (and the
same next step in `implementing`).

`change ship-status` already advertises `change check <id> --commit` for an
approved change (REQ-cmd-change-033). Handoff already calls uncommitted
edits under `affected_paths` `conditional` (REQ-change-093, invariant 42).
Neither stops review or ship, and neither is the `Next:` line after approve.

This is READY: it adds and changes requirement text. CHANGE-7 already says
the tool must name the single next thing to do; the criteria below come from
the issue and the later re-measurement, not from new human-intent wording.

Out of scope, deliberately: making `--commit` the default for `change check`
(that would change REQ-cli-args-010; bare check stays valid for a first
pass), and restoring the refuted unrecoverable-hook refusal.

## From the change's design.md

# Design

Two coordinated gates, both taken from the remaining #535 defect.

## Guided next action (REQ-change-156)

The domain `next_action` for an approved or implementing change whose
approval is current names `specsync change check <id> --commit`, in both
text and JSON, for `change status` and `change show`. Open tasks still
come first (REQ-change-132); the command after them gains `--commit`.
REQ-change-132's first acceptance line is updated so it does not keep
pinning the bare command.

Bare `change check` without `--commit` stays a legal command
(REQ-cli-args-010). This change only fixes the guided path.

A verifying change that already names `change check` as its next step
names `--commit` on that command as well, so the same gap does not remain
when verification is not current.

## Review and ship refuse an unbound verification (REQ-change-157)

`change review` and `change ship` refuse when the recorded verification
commit does not contain the change's own work: the current contents of
its `affected_paths`, and any canonical specs the change materialized.
The refusal names that commit and `specsync change check <id> --commit`.
No review is recorded. Nothing is finalized or archived.

"The change's own work" has to include affected paths, not only
materialized specs: the #535 reproduction is a no-spec documentation
change, and a specs-only check would have missed it.

Uncommitted files under `.specsync/` alone do not trigger this refusal.
`review.json` is uncommitted between review and finalize by design
(REQ-change-093, invariant 42).

`change ship` finalizes, so the same refusal belongs in the shared
finalization preflight; otherwise `change finalize` would bypass it.
That is the implementation of the issue's "review and ship" rule, not a
wider criterion.

After `change check --commit` records verification against a commit that
contains the work, review and ship proceed as they do today.

## Invariant 59

Claimed as the next free change.spec invariant (52–58 are taken by open
PRs; 51 is the last one on main). Wording to insert in numeric place
after definition approval:

> 59. After definition approval, the guided next action for an approved
> or implementing change names `specsync change check <id> --commit`.
> `change review` and `change ship` refuse when the recorded verification
> commit does not contain the change's own work — the current contents of
> its `affected_paths` and any canonical specs it materialized — and they
> do not record a review bound to such a commit. Uncommitted files under
> `.specsync/` alone do not trigger that refusal (#535, REQ-change-156,
> REQ-change-157).

## Not in this change

- `--commit` does not become the default for `change check`.
- The refuted August hook refusal ("historical integrity is invalid,
  found 0") is not restored.

## From the change's testing.md

# Testing

These cases are the #535 reproduction on a no-spec documentation change
whose affected path is modified and uncommitted. They will be written as
integration tests through the shipped binary, red first, after definition
approval. They have not been implemented in this draft.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-156 | After approve, text and JSON `change status` / `change show` name `specsync change check <id> --commit` and do not name a bare `change check` as the next command. The same holds in `implementing`. With one open task, the task is named first and the command that follows is `change check <id> --commit`. |
| REQ-change-157 | After a passing bare `change check`, `change review` exits non-zero, records no review, and does not bind a pass to the seed commit. `change ship` exits non-zero and does not finalize or archive. After `change check --commit` records verification against a commit that contains the affected-path work, review and ship may proceed. A tree whose only uncommitted files are under `.specsync/` is not refused by this rule. |
| REQ-change-132 | The open-task status case still names the task first; only the following check command gains `--commit`. |

The August unrecoverable-hook claim is not tested: it was re-measured as
absent and is out of scope.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
