## MODIFIED

### REQUIREMENT REQ-cmd-change-check-scoped-001

`specsync change check` SHALL run scoped verification for one change and SHALL NOT
invoke full archive terminal-evidence revalidation.

When `--commit` is set, after a successful first verification the command SHALL
commit any materialized working-tree changes, re-run scoped verification against
the committed tip, and commit the resulting verification evidence. A failed first
verification SHALL leave the git history unchanged.

When `--push` is set without `--commit`, the command SHALL fail before running
verification. When both are set, a successful commit sequence SHALL end with
`git push`.

Acceptance Criteria

- `change check --commit` leaves recorded verification evidence that
  `change audit --strict` accepts for that change when the tree is otherwise clean.
- A failing first verification produces no new commits.
- `--push` without `--commit` fails with an error naming the requirement.
