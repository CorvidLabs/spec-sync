# Lesson bundle — let-the-release-validator-accept-the-action-archive-gate-in-ci-and-docs-before-its-release

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let the release validator accept the Action archive gate in CI and docs before its release
- **Kind**: Operations
- **Paths**: .github/scripts/validate-release-version.py, .github/scripts/test-validate-release-candidate.py
- **Acceptance**: validate-release-version.py finds the packaged Action consumer step by its version input while action-consumer also runs the archive gate; documented steps that set require-archived or archive-gate-only may pin @main only while the published Action default is 6.1.1 or older; every other documented step still must pin the published release; test-validate-release-candidate.py passes and proves the allowance expires.

## Evidence

- Verification commit: `5e19021225de8d56547ed8f7d8a457476f3547d8`
- Base commit: `a5043451fe1cde489347f036d209dee088292310`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Companion to `run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived`,
on the same pull request. Running the release validator suite before ship found two collisions
with that change. The `action-consumer` job now runs the Action twice (the packaged binary, then
the archive gate against an open change), and `find_uses_step` needs exactly one `./` step. The
Action docs show the archive gate pinned to `@main`, because published `@v6.1.1` ignores the new
inputs, while the validator requires every documented step to pin the published release.

Ruled out: pinning the docs example to `@v6.1.1`, because a copied example would pass without
gating anything; and dodging the validator with a non-YAML fence.

## From the change's testing.md

# Testing

- `python3 .github/scripts/validate-release-version.py` passes on the tree.
- `python3 .github/scripts/test-validate-release-candidate.py` passes, including
  `test_archive_gate_main_ref_expires_once_a_newer_release_is_published`,
  `test_main_ref_without_archive_gate_inputs_is_rejected`,
  `test_ci_still_requires_candidate_binary` and `test_ci_candidate_mirror_remains_required`.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
