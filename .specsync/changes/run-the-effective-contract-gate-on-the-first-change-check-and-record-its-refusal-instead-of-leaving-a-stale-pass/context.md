---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: context
---

# Context

Reported downstream as Chough BUG-0003. `change check` records `passed: true` for a tree that
every later lifecycle gate refuses. A failed `change check --commit` then leaves that pass
current, so `change status` keeps pointing at review.

## Reproduction (released SpecSync 6.1.1)

The reporter reproduced this in a disposable copy of Chough's `examples/shipping` with a
synthetic approval.

1. `.specsync/config.toml` has `enforcement = "strict"`. Approve a change for module `shipping`,
   then add an undocumented `pub fn` to a file `shipping` owns.
2. `specsync change check <id> --json` exits 0. `verification.json` records `passed: true` with
   commands `["specsync check --spec shipping"]`.
3. `specsync check --strict` fails on the undocumented export.
4. `specsync change check <id> --commit` exits 1 with "effective contract `shipping`: Undocumented
   export ...". It records no attempt: `verification-attempts.json` still holds one passed
   attempt.
5. `specsync change status <id>` still names `specsync change review <id> --reviewer <human>`, and
   `summary.strict_validation_required` is false.

## The report's premise does not hold: configured enforcement is not involved

`enforcement = "strict"` does not reject this tree.

Measured with installed 6.1.1 on a copy of `examples/shipping` plus an undocumented `pub fn`:
`specsync check` exits 0 under `strict`, `warn` and `enforce-new`. Only `specsync check --strict`
exits 1. That matches the code:

- `compute_exit_code` (`src/commands/mod.rs:1204`): `Strict` blocks on errors, and on warnings
  only when the `--strict` flag is passed.
- The `EnforcementMode` documentation (`src/types.rs:342`): "Warnings still pass unless `--strict`
  is given".
- Nothing under `src/change/` reads `enforcement`.

A throwaway unit test on origin/main f240d37 (scratch export, fixture approver, not committed)
ran with `enforcement = "warn"` and `enforcement = "strict"`. The results were identical:

| Step | Result under `warn` and `strict` alike |
|------|----------------------------------------|
| First `check_change` | `Ok(passed: true)`, 1 attempt, command `specsync check --spec auth`, state verifying, next action `change review`, `strict_validation_required` false |
| Second `check_change` | `Err("effective contract `auth`: 0/1 exports documented; effective contract `auth`: Undocumented export 'undocumented_probe' from src/auth.rs")`, still 1 attempt, latest still passed, next action still `change review` |
| `audit_project` | Reports the same two errors |
| Clean control | Both checks pass, 2 attempts |

## Cause

1. `validate_effective_contracts` (`src/change/collect_requirement_ids.rs:851`) promotes every
   unsuppressed validator warning to a hard error. This is documented at `:833` and narrowed by
   REQ-change-063's ignore-rule and unauthored-stub suppressions.
2. Its record filter (`:855`-`:861`) includes an active record only when
   `!record.canonical_applied || record.state == ChangeState::Verifying`. The filter arrived with
   audited reopening (e2184550, #353).
3. `change check` materializes before it verifies. `materialize_change_deltas` sets
   `canonical_applied = true` and `state = Implementing`
   (`src/change/sdd_version.rs:2948`-`:2949`). Only then does `verify_change_locked` run the gate
   (`:3040`). On the first check the gate therefore excludes the very change it is verifying, and
   with one active change it validates nothing. REQ-change-007 says "Effective-contract validation
   runs during verification and acceptance"; on the first verification it does not run for the
   change itself.
4. The scoped pass `evaluate_spec_code_sync` (`src/change/sdd_version.rs:3268`) fails on warnings
   only under explicit `--strict` (REQ-change-023). It records `passed: true` and moves the change
   to verifying (`:3075`-`:3076`).
5. Every later run sees `state == Verifying`, so the gate includes the change. It refuses at
   `:3040`, before `record_verification_attempt` (`:3075`), so nothing is recorded:
   - `verification.json` keeps the pass.
   - The project-input digest has not moved, so `verification_is_current` still holds and status
     points to review.
   - `change check --commit` fails in its first pass (`run_checked_commit`,
     `src/commands/change.rs:3983`) and commits nothing.
   - `finalize` refuses at acceptance (`src/change/historical_definition_digest_matches.rs:2226`).
   - `change audit` reports the error (`:4036`).

The branch name `0xleif/fix/change-check-honors-configured-strict` predates this measurement.
This change does not make `change check` honor configured enforcement. `research.md` explains why
that would be the wrong fix.

## The built-in strict classification

`change_requires_strict_validation` (`src/change/sdd_version.rs:3372`) hard-codes SpecSync's own
paths (`src/change.rs`, `src/mcp.rs`, `src/github.rs`, `.github/workflows/release.yml`,
`.github/workflows/post-merge-archive.yml`) and modules (`change`, `mcp`, `github`,
`authentication`, `security`). That is this repository's dogfooding baked into product code.

- Since REQ-change-091 it changes nothing about verification. It sets only the advisory
  `summary.strict_validation_required` (`src/change/historical_definition_digest_matches.rs:3466`),
  plus `verification_commands_for_change`, which is dead outside tests (`sdd_version.rs:3315`).
- `src/change.rs` has been a 5-line `include!` shim since #810, so that entry no longer matches
  the implementation under `src/change/`.
- This repository's own `.specsync/sdd.json` already lists the same files in `strict_paths`.
- It played no part in this defect: `shipping` is not classified, and the gate refuses
  regardless.

It is left unchanged here. Removing it, and correcting the docs sentence that says the
classification "adds validators" (`site/src/content/docs/workflow.md:84`,
`site/src/content/docs/cli.md:421`), is a separate change for the human to decide.

## Human intent

This change links to existing criteria and adds none:

- SHIP-1.a: the code is checked against the contract as it now stands.
- SHIP-1.d: the evidence matches the tree it describes.
- SHIP-3.a: finalizing needs verification that is still current.
- CHANGE-7: the single next thing to do.

CHECK-3 and CHECK-3.a (an undocumented export is a warning unless promoted) are unchanged for
`specsync check`.
