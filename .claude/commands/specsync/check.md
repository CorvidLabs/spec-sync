---
description: Run scoped SpecSync change verification for one change (materialize deltas + spec↔code sync)
argument-hint: [change-id]
---

Run scoped SpecSync change verification.

Arguments: `$ARGUMENTS`

1. Prefer `specsync change check $ARGUMENTS` when an id or partial id is provided; otherwise run `specsync change check`.
2. Expect **scoped** verification only — this change's materialization and in-process spec↔code sync. Do not run a full archive integrity walk.
3. Stream/wait for exit. On success, follow the printed **Next:** action (review, PR, or finalize path).
4. Do **not** run `specsync change audit` unless the user asked for project health over active workspaces and living specs.
