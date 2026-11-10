# Lesson bundle: the-release-tests-exercise-the-race-guards-in-the-shipped-profile

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The release tests exercise the race guards in the shipped profile
- **Kind**: BugFix
- **Specs**: cmd_generate, mcp, validator
- **Paths**: src/commands/generate.rs, src/mcp/confinement/tests.rs, src/validator/consumed_by_re.rs, fledge.toml, docs/RELEASING.md
- **Acceptance**: cargo test --release runs a unit test for each of verify_coverage_project_root, RetainedGenerateRoot::verify_public_path and ConfinedReadRoot::revalidate_before_success that replaces the root after it is retained and asserts the guard refuses; fledge lanes run release-candidate runs them under --release (CI-11)

## Evidence

- Verification commit: `633acf3d10a9296ef9d5623b1dcecce19cacbf83`
- Base commit: `baf6e45509ac1fd4d5cbedfdf87d4dc17805a195`
- Verified by: `specsync check --spec cmd_generate --spec mcp --spec validator`

## From the change's context.md

# Context

Issue #614 (p1). After #581 gated seven race tests to debug builds, three TOCTOU guards had
no test that could run under `cargo test --release`, and no pipeline runs release tests.
Leif approved CI-11 on 2026-11-06 ("When I run the release tests, the race guards in the
shipped profile are tested too"). It is recorded in `hi/ci.md` on this branch.

Each guard now has an in-crate unit test that retains the root, replaces the directory
under the same path, and asserts the guard refuses. The test controls the timing, so it
needs no debug-only rendezvous and compiles in every profile. The release-candidate lane
gains `test-release-guards` (`cargo test --release --locked --bin specsync --
release_race_guard`).

Not approved by the worker: this lands hi intent (CI-11) and changes the release-candidate
lane, so under Leif's 2026-11-10 merge rule it stops at a ready PR for his scope approval.

## From the change's design.md

# Design

The three guards compare a retained directory's identity (device and inode on Unix) with
the identity reopened through the public path or route. A test that renames the directory
away and creates a new one at the same path forces a different identity deterministically,
which is the state a lost race leaves. That reproduces the race outcome without timing,
so no debug-only rendezvous (#581) is involved and the tests compile under `--release`.

The tests are Unix-only, like the existing race tests; Windows is not a supported target
as of 6.0. They live beside each guard: a `release_race_guard_tests` module in
`src/validator/consumed_by_re.rs` and `src/commands/generate.rs`, and a test in
`src/mcp/confinement/tests.rs`. They share the `release_race_guard` name prefix, so one
filter selects them for the release-candidate lane.

## From the change's testing.md

# Testing

`cargo test --bin specsync release_race_guard` (debug) and `cargo test --release --locked
--bin specsync -- release_race_guard` both run the three tests.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| CI-11 | unit `release_race_guard_coverage_root_refuses_a_replaced_root`, `release_race_guard_generate_root_refuses_a_replaced_root`, `release_race_guard_mcp_read_root_refuses_a_replaced_directory` under `--release` |

## Where these lessons go

- `specs/cmd_generate/cmd_generate.3md`
- `specs/mcp/mcp.3md`
- `specs/validator/validator.3md`
