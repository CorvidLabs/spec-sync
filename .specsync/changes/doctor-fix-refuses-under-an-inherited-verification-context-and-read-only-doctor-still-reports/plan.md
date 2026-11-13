---
change: doctor-fix-refuses-under-an-inherited-verification-context-and-read-only-doctor-still-reports
artifact: plan
---

# Plan

## Approach

Add `Command::Doctor { fix: true, .. }` to the families the root dispatcher passes through
`reject_recursive_lifecycle_dispatch` before dispatch, and say so in REQ-cli-003, the cli
Purpose and `docs/HLD.md`.

## Out of scope

Plain `doctor` stays as it is under the marker. No other command joins the guard.

## Steps

1. A failing integration test: `doctor --fix`, `--fix --dry-run` and `--fix --close-late` each
   refuse once with the recursion diagnostic; plain `doctor` matches its unmarked run.
2. The dispatcher guard.
3. `docs/HLD.md` names `doctor --fix` in its guard line.

## Risks

A caller that runs `doctor --fix` under a verification context now gets exit 1. That is the
intended refusal; nothing in SpecSync sets the marker on its own children (REQ-change-023).

## Constraints consulted

REQ-change-023 (verification runs no configured command), kept as the contract pin
`cli_contracts_describe_an_inherited_verification_context_not_a_child` checks.
