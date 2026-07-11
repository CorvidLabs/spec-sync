---
spec: cmd_lifecycle.spec.md
---

## Tasks

- [x] Implement lifecycle status commands and guard evaluation
- [x] Add CI-oriented lifecycle enforcement command

## Gaps

- History storage behavior is primarily covered at helper level; broader CLI tests should exercise enabled/disabled history modes together.

## Post-5.0 Test Debt

- [ ] Add end-to-end CLI coverage for lifecycle history output when `track_history` is enabled

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
