---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: requirements
---

# Requirements

## REQ-cmd_change-check-commit

`specsync change check --commit` SHALL run scoped verification, commit any
materialized tree changes, re-verify against the committed tip, and commit
verification evidence, so the resulting HEAD is a state `change audit --strict`
accepts for that change.

## REQ-cmd_change-check-push

`specsync change check --push` SHALL require `--commit` and, after a successful
`--commit` sequence, run `git push`.

## Acceptance

- First verification failure commits nothing.
- Successful `--commit` leaves a clean tree and recorded verification at HEAD.
- `--push` without `--commit` fails with a clear error naming the requirement.
