## ADDED

### REQUIREMENT REQ-github-026

Every pull request to main SHALL start CI, and the path classifier alone SHALL pick its lane.

- `.github/workflows/ci.yml` SHALL NOT filter its `pull_request` or `push` trigger by path (`paths:` or `paths-ignore:`). Both triggers keep `branches: [main]`.
- `.github/scripts/classify-ci-paths.sh` SHALL decide which jobs a started run selects. A path it does not name goes to the full lane, as before.
- So a pull request that changes only paths outside the old filter, such as `.chough/` records or `scripts/`, SHALL still get the required `Required CI gate` check (CI-6).

#### Acceptance Criteria

- A test fails when either trigger in `ci.yml` has a `paths:` or `paths-ignore:` filter, and its message names the recovery.
- The real classifier puts a change that touches only Chough records into a lane, and so does a change that touches only `scripts/`. Simulating that lane with the required-gate model of `test-required-ci-gate.py` reports `Required CI gate` green on both `pull_request` and `push` when every selected job passes.
- The fledge `verify` and `ci` lanes and `ci.yml` run that test, and the test fails when they stop.
