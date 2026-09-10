---
description: Create and guide a verified spec-sync SDD change through its deterministic interview
argument-hint: <change-description>
---

1. SDD is opt-in via `specsync change adopt`. If change commands fail because the workflow is off, adopt only when the user asked to turn SDD on.
2. Run `specsync change new "$ARGUMENTS" --json`.
3. Read the returned `questions` array and interview the user one question at a time.
4. Record each answer with `specsync change answer <id> <question-id> "<answer>" --json`.
5. Continue until the question list is empty, then show the selected artifacts and next action.
6. Do not run `specsync change approve <id> --actor "<identity>"`, implement, `specsync change check <id> --commit`, `specsync change review <id> --reviewer "<identity>"`, or `specsync change ship`/`finalize` until that gate is reached. Do not use v1 `start`/`verify`/`accept`/`archive` as the happy path.
7. After implementation, run `specsync change check <id> --commit` (or the installed check command with --commit when preparing ship evidence). Use `specsync change audit` only for active-workspace project health. Never expect check to rewalk archived terminal evidence.
