---
change: a-refused-check-records-its-refusal
artifact: context
---

# Context

Issue #847: when `change check` refuses before verification starts (open
tasks, missing requirement evidence, an unattributed gate refusal), it exits 1
and records nothing — no `verification-attempts.json` entry, no reason
anywhere in the workspace. A reader (Chough's `next`, a later session) cannot
tell the check was tried or why it failed.

The comment at the gate says the early refusal is deliberate, but the reason
it gives — "refused before any attempt is recorded" — is the defect. The fix
appends a `passed: false` attempt carrying the refusal reason to
`verification-attempts.json` only: `verification.json` (the last completed
verification) is never overwritten by a refusal. The attempt gains an optional
`refusal` field; absent in every prior record, so old bytes stay valid.
