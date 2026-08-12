---
id: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
state: archived
type: refactor
base_commit: 3f121c9205420f6f75b2f4a082433c03cf1949a5
---

# Make drift gate by default: flip the default enforcement mode from warn to strict so validation errors exit non-zero without an explicit flag

## Intent

Make drift gate by default: flip the default enforcement mode from warn to strict so validation errors exit non-zero without an explicit flag

## Affected Canonical Specs

- `types`

## Acceptance Criteria

- The default EnforcementMode SHALL be Strict, so that a validation error exits non-zero without an explicit flag; warnings SHALL continue to pass unless --strict is given; --enforcement warn SHALL remain available as the non-blocking opt-out; and the change SHALL be shipped separately from the trust-layer severing so the two exit-code changes remain independently bisectable.

## No-spec Rationale

Not applicable
