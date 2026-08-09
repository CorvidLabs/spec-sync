---
change: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-056 | Focused change-domain test proves malformed correction history is reported as invalid without exposing the parse payload. |
| REQ-cmd-change-009 | Command-module regression coverage proves the text integrity gate returns only the fixed safe diagnostic; sandbox drill `026-text-status-corrections.sh` exercises `show`, `status <id>`, and aggregate `status` against the built binary. |

Run `cargo test change::` and `cargo test commands::change::` during scoped verification.
Run sandbox drill `026-text-status-corrections.sh` with the built WIP binary; it must pass the
fixed fail-closed expectation.
