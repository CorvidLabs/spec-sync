---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: design
---

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
