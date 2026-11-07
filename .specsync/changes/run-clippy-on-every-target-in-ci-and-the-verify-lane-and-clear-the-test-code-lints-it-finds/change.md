---
id: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
state: draft
type: operations
base_commit: ab6882464fadb1bdfc71416d729534323e5a3836
---

# Run clippy on every target in CI and the verify lane, and clear the test-code lints it finds

## Intent

Run clippy on every target in CI and the verify lane, and clear the test-code lints it finds

## Affected Canonical Specs

- `github`
- `change`
- `validator`
- `merge`
- `hash_cache`
- `generator`
- `cmd_scaffold`
- `cmd_init`
- `agents`

## Acceptance Criteria

- The `github` spec gains REQ-github-028: CI's `test` job and the fledge `lint` task (which the `verify`, `ci`, `check`, `pre-commit` and `repo` lanes run) run `cargo clippy --all-targets -- -D warnings`, so the lint gate compiles and lints the test code as well as the shipped code (CI-8); the spec version, a testing row and the Change Log follow.
- `.github/scripts/test-clippy-all-targets.py` fails, naming the recovery, when any `cargo clippy` step in `ci.yml` or task in `fledge.toml` lacks `--all-targets` or `-D warnings`, or passes `-A`, `--allow` or `--cap-lints`; when CI's `test` job or the fledge `lint` task stops running clippy; when the `verify` or `ci` lane stops running `lint`; and when the `verify` and `ci` lanes or `ci.yml` stop running the test itself.
- `cargo clippy --all-targets -- -D warnings` exits 0 at the change's tip. The 36 lints it reports on main (ab688246), all in test code across 15 files, are fixed in the code: no new `allow` or `expect` attribute, no lint group lowered, no lint flag added, and no test weakened or removed (the constant assertion in `src/agents.rs` becomes a compile-time `const` assertion). The 28 existing `allow(clippy::...)` attributes are unchanged.
- `hi/ci.md` gains CI-8 in Leif's words, "When the pipeline says the code lints clean, that includes the tests.", with its source in the Intent section, and `INTENT.md` is regenerated with `hi index`.
- Failing test first: on the unmodified tree the new test fails (neither CI's clippy step nor the fledge `lint` task has `--all-targets`, and the test is not wired into fledge or CI) and `cargo clippy --all-targets -- -D warnings` fails with 36 errors; after the change both pass, and `cargo test` and `cargo clippy -- -D warnings` still pass. No tag or release is created or published.

## No-spec Rationale

Not applicable
