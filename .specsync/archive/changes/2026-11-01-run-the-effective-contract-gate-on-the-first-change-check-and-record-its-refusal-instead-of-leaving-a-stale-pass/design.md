---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: design
---

# Design

## Part A: the gate sees the change it is verifying

`verify_change_locked` passes the effective-contract gate a roster in which the change under
verification is treated as verifying. That is the state this attempt records. The gate then
validates that change's declared modules on every check, the first one included, by the same
rule finalization and audit apply. The filter in `validate_effective_contracts` stays as it is for
every other caller.

## Part B: a refusal of a declared module is recorded, not just returned

`EffectiveContractOutcome` gains private per-module attribution, so verification can tell two
kinds of refusal apart:

- A refusal of a module this change declares is a spec↔code verdict about this change. It is
  folded into the attempt.
- Every other gate failure stays a pre-attempt refusal, exactly as today. That covers ordering,
  workspace creation, an unresolvable canonical spec, or a module only another active change
  declares.

Order inside `verify_change_locked`:

1. Approval, definition, delta, dependency and conflict checks. Unchanged.
2. Effective-contract gate. Non-attributable errors return at once, as today. Errors for declared
   modules are held.
3. Tasks and requirement-evidence completeness. Unchanged pre-attempt refusals.
4. The scoped pass. Unchanged rule and unchanged command string.
5. One attempt is recorded:
   - `commands[0]` is the scoped entry.
   - When the gate held errors, `commands[1]` is
     `CommandEvidence { command: "specsync change check <id> (effective contract: <m>, …)", success: false, exit_code: Some(1) }`,
     with modules sorted.
   - `passed` stays the conjunction of the entries.
6. State moves to verifying, as for any recorded attempt. On failure the error is today's
   `verification failed: …; see commands[] in …/verification.json`, with the gate's findings in
   the summary.

## Resulting behavior

| Situation | origin/main f240d37 | After |
|-----------|---------------------|-------|
| First check, declared module has an undocumented export | `passed: true`, exit 0, next `change review` | `passed: false`, exit 1 naming the export, next `change check` |
| Later check or `--commit`, same tree | exit 1, nothing recorded, pass stays current | exit 1, failed attempt recorded, pass superseded |
| Clean declared modules | one scoped command, pass | identical |
| `--strict` | scoped command with `--strict` | identical; the gate entry is added only when the gate refuses |
| Warning only in a path-scoped spec, or a `--no-spec-change` change | not gated by the effective contract | identical |
| Gate failure caused only by another change's module | pre-attempt refusal | identical |
| `enforcement` = warn, enforce-new or strict | irrelevant to the gate | still irrelevant |

## Contract edits (see `deltas/change.md`)

- ADDED REQ-change-112: the gate runs on every check, and its refusal is recorded.
- MODIFIED REQ-change-023: the evidence bullet names the gate entry.
- MODIFIED SPEC SECTION Contract, item 5: the same, so the spec does not keep a sentence that the
  new evidence contradicts.

## Compatibility

- Persisted shape is unchanged, because `commands` is already a list. A failed attempt may carry
  two entries.
- Readers that assume one entry, or that a pass is the only first-check outcome, see a new case.
- Recorded passes from 6.1.1 stay readable and stay current until the next check. If the gate
  refuses that check, it records a failure that supersedes them.
- No digest, schema version or archive changes.
- `summary.strict_validation_required` and `change_requires_strict_validation` are untouched.
