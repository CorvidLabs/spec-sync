---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: docs
---

# Docs

- `specs/change/change.spec.md`: `validate_retirement` Public API row updated to describe
  recorded-evidence authentication and custody persistence instead of current recursive
  replacement freshness; changelog entry and version bump.
- `specs/change/change.3md`: new lesson plane recording that receipt classifications are
  historical facts and custodian freshness is the custodian's own reported state.
- No user-facing CLI surface changes: commands, flags and output shapes are unchanged; the
  staleness remedy text now names a recovery that actually runs.
