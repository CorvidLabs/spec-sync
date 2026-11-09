# Lesson bundle: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run clippy on every target in CI and the verify lane, and clear the test-code lints it finds
- **Kind**: Operations
- **Specs**: github, change, validator, merge, hash_cache, generator, cmd_scaffold, cmd_init, agents
- **Paths**: .github/workflows/ci.yml, fledge.toml, .github/scripts/test-clippy-all-targets.py, CONTRIBUTING.md, .github/PULL_REQUEST_TEMPLATE.md, docs/ci-confidence.md, hi/ci.md, INTENT.md, specs/github, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs, src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs, src/change_tests/substantive_plan.rs, src/change_tests/fast_archived_reads.rs, src/change/sdd_version.rs, src/change/consider_accepted_evidence_anchor.rs, tests/integration/change.rs, src/validator/tests.rs, src/merge_driver.rs, src/hash_cache.rs, src/generator.rs, src/commands/scaffold.rs, src/commands/init.rs, src/agents.rs
- **Acceptance**: The `github` spec gains REQ-github-028: CI's `test` job and the fledge `lint` task (which the `verify`, `ci`, `check`, `pre-commit` and `repo` lanes run) run `cargo clippy --all-targets -- -D warnings`, so the lint gate compiles and lints the test code as well as the shipped code (CI-8); the spec version, a testing row and the Change Log follow.
- **Acceptance**: `.github/scripts/test-clippy-all-targets.py` fails, naming the recovery, when any `cargo clippy` step in `ci.yml` or task in `fledge.toml` lacks `--all-targets` or `-D warnings`, or passes `-A`, `--allow` or `--cap-lints`; when CI's `test` job or the fledge `lint` task stops running clippy; when the `verify` or `ci` lane stops running `lint`; and when the `verify` and `ci` lanes or `ci.yml` stop running the test itself.
- **Acceptance**: `cargo clippy --all-targets -- -D warnings` exits 0 at the change's tip. The 36 lints it reports on main (ab688246), all in test code across 15 files, are fixed in the code: no new `allow` or `expect` attribute, no lint group lowered, no lint flag added, and no test weakened or removed (the constant assertion in `src/agents.rs` becomes a compile-time `const` assertion). The 28 existing `allow(clippy::...)` attributes are unchanged.
- **Acceptance**: `hi/ci.md` gains CI-8 in Leif's words, "When the pipeline says the code lints clean, that includes the tests.", with its source in the Intent section, and `INTENT.md` is regenerated with `hi index`.
- **Acceptance**: Failing test first: on the unmodified tree the new test fails (neither CI's clippy step nor the fledge `lint` task has `--all-targets`, and the test is not wired into fledge or CI) and `cargo clippy --all-targets -- -D warnings` fails with 36 errors; after the change both pass, and `cargo test` and `cargo clippy -- -D warnings` still pass. No tag or release is created or published.

## Evidence

- Verification commit: `0cfc360996b13412ac3f536802a9ff9f9149cc50`
- Base commit: `ab6882464fadb1bdfc71416d729534323e5a3836`
- Verified by: `specsync check --spec agents --spec change --spec cmd_init --spec cmd_scaffold --spec generator --spec github --spec hash_cache --spec merge --spec validator`

## From the change's context.md

# Context

Issue #608 (P1, G-608 in the 2026-11-06 triage): CI's `test` job ran `cargo clippy -- -D warnings` (`ci.yml` line 241), and so did the fledge `lint` task (`fledge.toml` line 14). Without `--all-targets`, clippy compiles only the library and binary targets, so it never lints a `#[cfg(test)]` module or `tests/`. A green lint run meant that the shipped code lints clean, but it read as the code linting clean.

Measured on main ab688246 with the pinned toolchain (1.89): `cargo clippy --all-targets -- -D warnings` exits 101 with 36 errors. 35 are in the binary's test target and 1 is in `tests/integration`, across 15 files. #608 counted 20 at ecff965e, so test code kept accumulating lints nobody saw. All 36 are in test-only code: plain `cargo clippy -- -D warnings` passes.

HI: CI-8 is new, in Leif's words: "When the pipeline says the code lints clean, that includes the tests." (Herdr orc tab, 2026-11-06, choice "A", relayed by orc t18). REQ-github-028 was claimed with the orc.

Ruled out: `allow`/`expect` attributes, lowering a lint group, or passing `-A`/`--cap-lints`. Each would turn the gate green by looking away, which is the defect class #608 names.

## From the change's design.md

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

## From the change's testing.md

# Testing

| Case | Expected | How |
|---|---|---|
| A clippy step or task without `--all-targets` | The test fails and names the recovery | `test_every_ci_clippy_step_lints_every_target_and_denies_warnings` and `test_every_fledge_clippy_task_lints_every_target_and_denies_warnings` fail on the unmodified `ci.yml` and `fledge.toml` |
| A clippy command that lets lints through (`-A`, joined `-A…`, `--allow`, `--cap-lints`, no `-D warnings`) | Caught | `test_the_problem_check_catches_each_weakening` feeds each form to the rule; the joined form failed before its Codex round-1 fix |
| The lint gate is removed | The test fails | CI's `test` job must run clippy; `lint` must exist and run in `verify` and `ci` (`test_the_verify_and_ci_lanes_run_the_lint_task`) |
| The test stops running | The test fails | `test_this_test_runs_through_fledge_and_ci` |
| The test code lints clean | `cargo clippy --all-targets -- -D warnings` exits 0 | Fails with 36 errors on ab688246 and passes at the tip; `cargo test` and `cargo clippy -- -D warnings` still pass |
| No lint is weakened | No new `allow` or `expect` | `git diff origin/main` adds no `allow(`/`expect(` attribute; the `allow(clippy::` count stays at 28 |

Failing first: on the unmodified tree the lockstep test fails 3 tests (CI step, fledge task, not wired in), and `cargo clippy --all-targets -- -D warnings` exits 101 with 36 errors.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-028 | `.github/scripts/test-clippy-all-targets.py`: `test_every_ci_clippy_step_lints_every_target_and_denies_warnings`, `test_every_fledge_clippy_task_lints_every_target_and_denies_warnings`, `test_the_verify_and_ci_lanes_run_the_lint_task`, `test_the_problem_check_catches_each_weakening`, `test_this_test_runs_through_fledge_and_ci`; `cargo clippy --all-targets -- -D warnings` exit 0 at the tip |

## Where these lessons go

- `specs/github/github.3md`
- `specs/change/change.3md`
- `specs/validator/validator.3md`
- `specs/merge/merge.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/generator/generator.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
- `specs/cmd_init/cmd_init.3md`
- `specs/agents/agents.3md`
