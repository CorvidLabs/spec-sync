## ADDED

### REQUIREMENT REQ-github-028

The lint gate SHALL lint the test code as well as the shipped code.

- CI's `test` job and the fledge `lint` task SHALL run `cargo clippy --all-targets -- -D warnings`. The `verify` and `ci` lanes run that task, and so do `check`, `pre-commit` and `repo`.
- No `cargo clippy` step in `ci.yml` or task in `fledge.toml` SHALL drop `--all-targets` or `-D warnings`. None SHALL pass `-A`, `--allow` or `--cap-lints`.
- So a clean lint result in the pipeline means the tests lint clean too (CI-8).

#### Acceptance Criteria

- A test fails, naming the recovery, when any `cargo clippy` step in `ci.yml` or task in `fledge.toml` lacks `--all-targets` or `-D warnings`, or passes `-A`, `--allow` or `--cap-lints`.
- The test fails when CI's `test` job stops running clippy, when `fledge.toml` has no `lint` task that runs it, or when the `verify` or `ci` lane stops running `lint`.
- The fledge `verify` and `ci` lanes and `ci.yml` run that test, and the test fails when they stop.
- `cargo clippy --all-targets -- -D warnings` exits 0 on the tree that carries the requirement.
