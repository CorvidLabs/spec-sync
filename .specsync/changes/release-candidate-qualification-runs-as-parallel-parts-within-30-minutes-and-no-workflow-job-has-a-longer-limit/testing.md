---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: testing
---

# Testing

- `.github/scripts/test-validate-release-candidate.py`: the `parts` mode accepts ten good records and writes two successful platform files. It refuses a missing, duplicated, unexpected or failed part, a part bound to another tag, SHA, version or workflow revision, slices that report different whole-list counts, and slice counts that do not add up. The `qualify` matrix, its limit and its pins are held to the workflow file.
- `.github/scripts/test-workflow-time-limits.py`: every workflow job declares a limit of at most 30 minutes, and the test fails on a copy with a limit removed or raised.
- The rehearsal lockstep test: the rehearsal's parts, images, pins, commands and limit equal the release workflow's.
- The part script on this machine: one slice run for real with the pinned cargo-nextest, and its record compared with the listing.
- The rehearsal on the pull request, on hosted Ubuntu and macOS: each part's time against the 30-minute limit, and the slice counts adding up.
- `actionlint` on the changed workflows, the release validators, the full native Verify lane, `change audit` and `check --strict`.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-007 | `.github/scripts/test-validate-release-candidate.py`: the existing candidate, evidence and release-chain tests, and the `qualify` matrix pins |
| REQ-github-031 | `.github/scripts/test-validate-release-candidate.py`: the `parts` mode tests; the rehearsal run on the pull request |
| REQ-github-032 | `.github/scripts/test-workflow-time-limits.py` |
| REQ-github-033 | `.github/scripts/test-validate-release-candidate.py`: the rehearsal lockstep test; the rehearsal run on the pull request |
