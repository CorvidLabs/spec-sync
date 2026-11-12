# Lesson bundle: run-the-trust-ci-job-on-ubuntu-24-04-so-the-augur-risk-gate-can-start

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run the trust CI job on ubuntu-24.04 so the augur risk gate can start
- **Kind**: BugFix
- **Paths**: .github/workflows/trust.yml
- **Acceptance**: The trust CI job runs on an ubuntu-24.04 runner, so the CorvidLabs Trust gate's augur risk step starts (no libxml2.so.2 exit-127), while every other job's runner is unchanged; the workflow comments name CorvidLabs/augur#59 as the reason and the revert condition

## Evidence

- Verification commit: `57d0f6489ab3002c07484230e614a4151ddaedcb`
- Base commit: `b2ec548ff9fba5f9308a3c7fc61cc853ecd3d0dd`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

GitHub is rolling `ubuntu-latest` over to `ubuntu-26.04`. On 2026-11-11 this
repository's `trust` job failed twice (run 43099930195, 21:08 and 21:17 UTC):
the CorvidLabs Trust gate's risk step downloads the prebuilt
`augur-linux-x86_64` binary, which links `libxml2.so.2` via Swift
FoundationXML, and 26.04 ships libxml2 2.15 under the new SONAME
`libxml2.so.16` — there is no `libxml2.so.2` to install. The binary exits 127
before doing any work. The same workflow passed hours earlier on an
`ubuntu-24.04` runner image (run 43092410912, 20:00 UTC), so the runner OS is
the only variable.

`apt install libxml2` was ruled out: no 26.04 package provides the old SONAME.
The durable fix is upstream (CorvidLabs/augur#59: a static or
libxml2.so.16-linked rebuild); until then, the trust job pins
`runs-on: ubuntu-24.04`, which still ships `libxml2.so.2`. The comment on the
pin names the issue and the revert condition.

Only the trust job changes. Every other job keeps its runner; the Archive job
in the same workflow does not run augur and stays on `ubuntu-latest`.

## From the change's testing.md

# Testing

A workflow-only change has no unit tests. Verification is the pull request's
own CI:

1. The `trust` workflow run on this PR's branch shows `Image: ubuntu-24.04`
   in the trust job's "Set up job" log.
2. The trust job passes end to end, including the CorvidLabs Trust gate's
   augur risk step (no `libxml2.so.2` exit 127).
3. `git diff` on the branch touches only `.github/workflows/trust.yml`, and
   the only changed line is the trust job's `runs-on:` plus its comment.

The failure mode being fixed was observed twice on run 43099930195 with
`Image: ubuntu-26.04`; the pass condition above was the behavior on
`ubuntu-24.04` runners earlier the same day (run 43092410912).

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
