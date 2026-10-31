---
id: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
state: draft
type: bug_fix
base_commit: f240d379b9f023f5766e5c9b39508eb84e03e2bb
---

# Run the effective contract gate on the first change check and record its refusal instead of leaving a stale pass current

## Intent

Run the effective contract gate on the first change check and record its refusal instead of leaving a stale pass current

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- For an approved change whose declared module carries an unsuppressed validator warning (for example an undocumented export), the first `specsync change check <id>` after approval exits non-zero and names that warning. That is the verdict a second `change check`, `change check --commit`, `change finalize` and `change audit` already reach on the same tree. It is identical with `enforcement` set to `warn`, `enforce-new` or `strict`, because configured enforcement is not what refuses it.
- That refusal is recorded as an append-only failed attempt. `verification.json` reports `passed: false` and `verification-attempts.json` gains the attempt. `commands[]` keeps the scoped `specsync check --spec <name> …` entry and adds one entry with `success: false` whose command is `specsync change check <id> (effective contract: <module>, …)`, naming the gate and each refused declared module instead of claiming a `specsync check --strict` run.
- When a passed verification is already recorded and a later `change check` or `change check --commit` is refused by that gate for a declared module, the new failed attempt supersedes it. `change status` then names `specsync change check <id>` rather than `specsync change review`, and `--commit` commits nothing.
- A change whose declared modules carry no unsuppressed warning passes its first and later checks and records exactly the single scoped command it records today, without `--strict`. Explicit `--strict` behaves as today.
- Unchanged: `specsync check` and configured `enforcement` semantics; the gate's warning promotion and its ignore-rule and unauthored-stub suppressions; specs in scope only through `affected_paths`; `--no-spec-change` changes; gate failures not attributable to a module this change declares, which are still refused before any attempt is recorded; the acceptance and audit gates; accepted and archived evidence; `summary.strict_validation_required` and the built-in strict path and module classification.
- Regression tests fail on origin/main f240d37 and pass with the fix. They include discriminators for the first-check verdict under `warn` and under `strict`, for the recorded failure, and for the superseded pass; labelled controls for every unchanged case; and an integration test through the built binary covering `change check`, `change check --commit` and `change status --json`.

## No-spec Rationale

Not applicable
