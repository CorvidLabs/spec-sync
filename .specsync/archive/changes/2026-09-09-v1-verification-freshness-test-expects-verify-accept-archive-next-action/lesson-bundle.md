# Lesson bundle — v1-verification-freshness-test-expects-verify-accept-archive-next-action

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: V1 verification freshness test expects verify accept archive next_action
- **Kind**: BugFix
- **Specs**: change
- **Paths**: tests/integration/change.rs
- **Acceptance**: verification_freshness_status_and_check_are_environment_independent passes; next_action is the v1 verify/accept/archive string in local, ci, and github environments both before and after the governed input moves

## Evidence

- Verification commit: `edf45f036c73a7afeec3fb2e25cb9185d1a2fe25`
- Base commit: `418fa117e78ad1f65f462f3fc96ef0491d0136d2`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

CI `cargo test --verbose` now finishes the unit harness (2500 passed) and fails one integration test: `verification_freshness_status_and_check_are_environment_independent`.

The fixture writes `sdd.json` `"version": 1` and runs `change verify`. After the v1 Verifying next_action change it correctly reports verify → accept → archive. The test still expected v2 `review` / `check` strings.

Update the three expected next_action strings. Do not change product code.

## From the change's testing.md

# Testing

Fail: `cargo test --test integration -- verification_freshness_status_and_check_are_environment_independent` left = v1 verify/accept/archive, right = v2 review.

Pass: that test, plus `cargo test --test integration -- change::` if time allows. Unit `--bin specsync` already 2500 passed.

## Where these lessons go

- `specs/change/context.md`
