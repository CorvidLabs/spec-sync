---
change: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
artifact: requirements
---

# Requirements

## REQ-types-004 — drift gates by default

The default enforcement mode SHALL be strict, so a validation error exits non-zero without an
explicit flag. Warnings SHALL continue to pass unless `--strict` is supplied, and
`--enforcement warn` SHALL remain available as the non-blocking opt-out.
