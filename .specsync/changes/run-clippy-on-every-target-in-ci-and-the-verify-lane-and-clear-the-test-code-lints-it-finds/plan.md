---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: plan
---

# Plan

## Approach

Failing test first: the lockstep test, plus `cargo clippy --all-targets -- -D warnings` as the second failing check. Then clear the 36 test-code lints in the code. Then add `--all-targets` to CI and the fledge `lint` task, wire the test in, and record REQ-github-028 in the `github` spec.

## Out of scope

- Lints in other open PRs' new test code; each PR clears its own when it merges main.
- Adding lints, a `[lints]` table or `clippy.toml`.
- Changing the toolchain pin.
- Any other workflow or lane step.
- No tag, release or crates.io publish.

## Steps

1. Commit CI-8, the failing `test-clippy-all-targets.py` and this definition; record approval.
2. Fix the 36 lints file by file (research.md).
3. Change CI's clippy step and the fledge `lint` task to `--all-targets`, add the `clippy-all-targets-test` task to `ci`, `verify` and `repo`, and add the `ci.yml` `validate-action` step.
4. In `specs/github`: materialize REQ-github-028, bump the spec version, add a testing row, a Change Log row, and a lesson plane in `github.3md`. The eight owning specs' verbatim Purpose deltas materialize as a version bump plus this change's Change Log row each, with no section text changed.
5. Run `specsync change check --commit`, `specsync change audit --strict`, `fledge lanes run pre-push` and `fledge lanes run verify` (niced). Then review, ship, archive and push; CI must be green.

## Risks

- **Other open PRs.** Once this merges, an open PR whose new test code has a lint fails CI's `test` job after it merges main in. That is CI-8 working, but the queue should know: expect a one-line fix each.
- **Slower local lanes.** `pre-commit` and `check` now compile test code under clippy.
- **Overlap with open PRs.** Fixes touch files open PRs also touch (`sdd_version.rs`, the `change_tests` files), so a merge of main may conflict. Keeping each fix minimal keeps conflicts small.

## Constraints consulted

Never weaken a lint, gate, bound or test. The toolchain is pinned (1.89) so local and CI clippy agree. CI is archive-first. Workflow tests reuse the required-gate model's Ruby Psych loader. Builds run niced at `-j 4` on a shared machine.
