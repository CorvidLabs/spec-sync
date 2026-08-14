---
id: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
state: implementing
type: bug_fix
base_commit: 60360ed2c07f89f234b9484253826e7948bb6121
---

# Asking to view a module that does not exist must fail, not print nothing and succeed

## Intent

Asking to view a module that does not exist must fail, not print nothing and succeed

## Affected Canonical Specs

- `cmd_view`

## Acceptance Criteria

- Requesting a spec module that does not exist reports the unknown name and exits non-zero, instead of printing nothing and exiting zero, in both the human and machine-readable outputs. A near-miss name is answered with the module it probably meant; when nothing is close, the modules that do exist are listed. A spec that fails to render also causes a non-zero exit rather than being printed to stderr and ignored. Requesting a module that does exist still renders it and exits zero, and running with no filter at all is unchanged.

## No-spec Rationale

Not applicable
