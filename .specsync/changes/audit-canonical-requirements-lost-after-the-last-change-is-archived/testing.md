---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: testing
---

# Testing

Run CLI integration cases against a real base and branch. Archive the last change before removing the unrelated requirement. Require default audit, strict JSON audit and archive audit to report the same ID, file and restore guidance once. Confirm the audits leave the Git tree untouched.

A valid archived removal on the branch passes. A pre-existing archive containing the same removal still yields a reported loss. Moving the block between canonical files passes. Exercise a shallow checkout with a default ref and no merge base, then one without a default ref. Assert distinct diagnostics and the proposed exit behavior.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-archived-requirement-loss-001 | Planned archive-only and shallow-checkout scenarios in tests/integration/vanished_requirements.rs |
