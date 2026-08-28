---
id: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
state: implementing
type: operations
base_commit: 4b72b09de0e950b7a0479463dbefcac33d516cac
---

# Drop Windows from the release qualification lane and the release validator, and state that the retained Windows content guarantees are now unverified

## Intent

Drop Windows from the release qualification lane and the release validator, and state that the retained Windows content guarantees are now unverified

## Affected Canonical Specs

- `cmd_issues`
- `github`

## Acceptance Criteria

- The release qualify matrix contains only ubuntu and macos; REQUIRED_PLATFORMS is ('ubuntu','macos'); no Windows-only step remains in release.yml; the validator self-test passes 50/50; docs/ci-confidence.md and CHANGELOG.md state that the retained cfg(windows) code is now compiled and run nowhere and that those guarantees are best-effort and unverified; and the CHANGELOG paragraph that argued for keeping the lane says the argument was correct and the risk is accepted rather than resolved.

## No-spec Rationale

The cmd_issues and github modules own the touched paths, so ownership is declared with --spec; the edits are CI configuration, a validator constant, a test-only cfg gate and prose, so no canonical spec text moves.
