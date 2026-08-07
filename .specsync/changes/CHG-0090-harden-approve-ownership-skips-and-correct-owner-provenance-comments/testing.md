---
change: CHG-0090-harden-approve-ownership-skips-and-correct-owner-provenance-comments
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-053` | Unit path: approve rejects undeclared-owner paths (existing test). Empty specs without `no_spec_change` now fail ownership validation with an explicit error. Never-closed correct-owner still gates on `ensure_definition_approval_valid`; comments document weaker provenance. |

## Manual

Adversarial review of the four product fixes (session 2026-08-06) identified the
skip/comment issues; this change implements the agreed hardenings.
