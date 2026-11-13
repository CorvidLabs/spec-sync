---
change: close-changes-that-will-not-ship
artifact: plan
---

# Plan

## Approach

Close a change by moving its package, as a late close does, into `.specsync/archive/closed/`
with a `close.json` record, adding no change state.

## Out of scope

Chough's own `migrate` (another repository); filters such as "every change whose code is on
main" (Leif chose explicit ids); the lesson the closed change taught, which is a next step named
in the output, not a gate.

## Steps

1. Red integration tests.
2. Generalize the late-close mover over its record.
3. close.rs, hooks, CLI.
4. Docs, Public API rows, invariants.
5. Deltas once the wording is checked; tests, fmt and clippy.

## Risks

The late-close refactor must keep its messages and rollback: its unit and integration tests run.
The materialized-text check compares whole canonical files with the default branch, so another
unmerged change editing the same spec also blocks the close; the message names exactly what to
restore.

## Constraints consulted

REQ-change-143 (late close), REQ-change-158 and the relay form, REQ-cmd-change-035 (doctor), the
send-back orphan rule (#1058).
