---
change: let-the-release-validator-accept-the-action-archive-gate-in-ci-and-docs-before-its-release
artifact: context
---

# Context

Companion to `run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived`,
on the same pull request. Running the release validator suite before ship found two collisions
with that change. The `action-consumer` job now runs the Action twice (the packaged binary, then
the archive gate against an open change), and `find_uses_step` needs exactly one `./` step. The
Action docs show the archive gate pinned to `@main`, because published `@v6.1.1` ignores the new
inputs, while the validator requires every documented step to pin the published release.

Ruled out: pinning the docs example to `@v6.1.1`, because a copied example would pass without
gating anything; and dodging the validator with a non-YAML fence.
