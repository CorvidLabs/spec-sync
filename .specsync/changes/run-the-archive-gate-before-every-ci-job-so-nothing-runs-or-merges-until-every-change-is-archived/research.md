---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: research
---

# Research

## What depended on CI running before archival

| Dependency | Where | Resolution |
|------------|-------|------------|
| Ship guidance: push product tip, wait for trust + implementation ready | `SHIP_TRUST_LOCAL_GUIDANCE`, `build_ship_trust`, `ship_stages` in `src/commands/change.rs` | rewritten for local verify → review → ship; HEAD check-runs first |
| Archive/review tips reuse a green parent | `select-ci-lane.sh`, tip-only classify in `ci.yml` | removed; whole-PR classification only |
| corvid-pet selected on `review_required` | `ci.yml` | `review_required` cannot be true past the gate; selects on `full` |
| Scoped review provenance `github_actions_check` | `src/change/sdd_version.rs` | a label stamped by `change review`; does not query CI, unaffected |
| Implementation gate excluded Archive so review could wait on it | `ci.yml`, `test-required-ci-gate.py` | Archive is now required by both gates |

## Alternatives considered

- Gate merge only and keep product-tip CI: CI still runs before archival, which the owner ruled out.
- Gate only other repositories: spec-sync would not follow the rule it ships.
- A new `specsync` subcommand for the gate: needs a release before anyone can use it and a binary
  download before the gate; the existing Python script needs neither.

## Skipped required checks

GitHub reports a job skipped by a failed dependency as `skipped`, and a skipped required check
passes branch protection. CI already requires the aggregate `Required CI gate` (`if: always()`),
which fails when Archive fails; the Action docs tell consumers to do the same.
