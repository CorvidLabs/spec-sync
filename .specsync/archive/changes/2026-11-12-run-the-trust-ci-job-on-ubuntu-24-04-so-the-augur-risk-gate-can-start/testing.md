---
change: run-the-trust-ci-job-on-ubuntu-24-04-so-the-augur-risk-gate-can-start
artifact: testing
---

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
