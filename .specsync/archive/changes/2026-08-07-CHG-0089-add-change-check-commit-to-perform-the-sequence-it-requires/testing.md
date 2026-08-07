---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cli-args-010` | CLI parse unit tests: `change check --commit --push` sets both flags; bare `change check` leaves both false. |
| `REQ-cmd-change-check-scoped-001` | Sandbox drill `035-check-commit.sh`: happy path audit-clean after `--commit`; failing first verify leaves HEAD unchanged; `--push` alone errors with `--push requires --commit`. |

## Unit

- Parse-only cases in `src/commands/change.rs` tests for the new flags.

## Integration

Sandbox drill `drills/035-check-commit.sh` (spec-sync-sandbox).
