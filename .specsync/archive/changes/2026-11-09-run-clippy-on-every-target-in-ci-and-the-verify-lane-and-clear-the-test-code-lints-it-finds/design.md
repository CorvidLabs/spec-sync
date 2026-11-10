---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: design
---

# Design

**One task, every lane.** The fledge `lint` task becomes `cargo clippy --all-targets -- -D warnings`. The `verify` and `ci` lanes run it, and so do `check`, `pre-commit` and `repo`. Changing the one shared task keeps every local lint run equal to CI's, so a local green can't mean less than CI's. The cost is that `pre-commit` now compiles test code in check mode. `pre-push` doesn't run clippy and stays fast.

**CI.** CI's `test` job step `cargo clippy -- -D warnings` becomes `cargo clippy --all-targets -- -D warnings`. The job already builds and runs the tests, so the extra compile reuses that work. `Required CI gate` keeps its existing `needs`; no job is added or renamed.

**Lockstep test.** `.github/scripts/test-clippy-all-targets.py` reads `ci.yml` through `test-required-ci-gate.py`'s loader, which uses Ruby Psych as the other workflow tests do, and reads `fledge.toml` with the same regexes as `test-ci-trigger-paths.py`.
- It finds every `cargo clippy` command and requires `--all-targets` and `-D warnings`, with no `-A`, `--allow` or `--cap-lints`.
- It also requires that the commands exist: clippy in CI's `test` job, a `lint` task, and `lint` in `verify` and `ci`. A deleted gate then can't pass for want of input.
- A self-check feeds the rule each weakening it must catch.
- The test runs as fledge task `clippy-all-targets-test` in the `ci`, `verify` and `repo` lanes, and as a `validate-action` step in `ci.yml`, next to the other workflow tests.

**Lint fixes** go in the code, one kind at a time (research.md). None adds an `allow` or `expect` attribute, and none loosens an assertion. `assert!(AGENT_ARTIFACT_TEMPLATE_VERSION >= 6)` becomes a `const` assertion, so the guard moves from run time to compile time and gets stronger.

**Deltas for the eight owning specs.** Definition approval needs a delta for every affected spec. The lint fixes touch test code owned by `change`, `validator`, `merge`, `hash_cache`, `generator`, `cmd_scaffold`, `cmd_init` and `agents`, so each must be declared, and none changes its contract. Each of those eight deltas is therefore `## MODIFIED` / `### SPEC SECTION Purpose`, restating the canonical Purpose verbatim. That is the same shape #994 used for `cli`. Applying it leaves the section byte-identical (checked by simulating the replacement on each spec at db722f89). Materialization then bumps each spec's version and appends this change's Change Log row, which is the record AGENTS.md asks for when a module's files change. No requirement text in those modules changes. Only `github` gains a requirement (REQ-github-028).
