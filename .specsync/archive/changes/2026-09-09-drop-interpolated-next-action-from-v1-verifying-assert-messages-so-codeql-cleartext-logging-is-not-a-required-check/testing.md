---
change: drop-interpolated-next-action-from-v1-verifying-assert-messages-so-codeql-cleartext-logging-is-not-a-required-check
artifact: testing
---

# Testing

Fail: CodeQL `rust/cleartext-logging` on the two `{next}` interpolations in `workflow_v1_verifying_next_action_names_verify_accept_archive`.

Pass: `cargo test --bin specsync -- workflow_v1_verifying_next_action`. Panic messages must not contain `{next}`.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-100 | `workflow_v1_verifying_next_action_names_verify_accept_archive` still asserts verify/accept/archive and rejects check/finalize; panic messages no longer interpolate `{next}` |
