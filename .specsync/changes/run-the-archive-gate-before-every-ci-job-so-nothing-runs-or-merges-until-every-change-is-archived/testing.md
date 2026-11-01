---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: testing
---

# Testing

- `python3 .github/scripts/test-required-ci-gate.py`: archive-first contract (Archive needs
  nothing, every job depends on it, both gates require it, an open change runs nothing and turns
  both gates red), the Archive job dogfoods `archive-gate-only`, and the Action runs its gate
  step first and skips every other step under `archive-gate-only`.
- `.github/scripts/test-classify-ci-paths.sh` and `test-archive-readiness.sh` still pass.
- CI `action-consumer`: the packaged Action passes `require-archived` on a clean consumer and
  blocks `archive-gate-only` once an open change is added.
- `cargo test --bin specsync -- github:: agents:: ship`: guidance, `Archive` trust relevance,
  installed agent text.
- `specsync check --strict --require-coverage 100 --force` and `change audit --strict`.
