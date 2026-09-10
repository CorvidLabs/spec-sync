Create a verified spec-sync SDD change.

Arguments: $ARGUMENTS

1. Run `specsync change new "$ARGUMENTS" --json`.
2. Read the returned `questions` array and interview the user one question at a time.
3. Record each answer with `specsync change answer <id> <question-id> "<answer>" --json`.
4. Continue until the question list is empty, then show the selected artifacts and next action.
5. Do not approve, implement, check --commit, review, or ship/finalize until the corresponding human gate or work stage is reached. Do not use v1 `start`/`verify`/`accept`/`archive` as the happy path.
6. After implementation, run `specsync change check <id> --commit` (or `/specsync:check` with --commit when preparing ship evidence). Use `specsync change audit` only for active-workspace project health. Never expect check to rewalk archived terminal evidence.
