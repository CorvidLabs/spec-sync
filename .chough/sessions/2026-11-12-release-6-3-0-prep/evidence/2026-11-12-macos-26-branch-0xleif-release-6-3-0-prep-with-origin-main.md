---
kind: evidence
id: 2026-11-12-macos-26-branch-0xleif-release-6-3-0-prep-with-origin-main
aliases: [OBS-structural-a1f553d651e876c963f0b38a]
claim: "macOS 26, branch 0xleif/release/6.3.0-prep with origin/main 9e7abb0f7 (#1067) merged in. These pass at Cargo version 6.3.0: validate-release-version.py ('Validated release version 6.3.0 across current distribution surfaces'), test-validate-release-candidate.py (OK, 67 tests) and validate-workflow-runtime-pins.py. CHANGELOG [6.3.0] now carries #1067's relayed-decision entry: workflow version 3; once a repository records one relayed decision, every machine that runs change commands there needs 6.3.0 or later; workflow-v1 relays are not gated. Diff review: the action.yml default, PUBLISHED_ACTION_DEFAULT, the docs default row, current-install guidance and the immutable @v6.2.0 prose stay at 6.2.0."
result: passed
evidence_kind: structural
method: automated
revision: a8bf6156b2a6caf5704d87ea44d1dafb4f4c4fb9
content: 50e7901c173f30e9deb1e11c0abe9d670421b0fd2ffea0b295f6fb3d4f5ae207
environment: local
criteria: [CI-1.a, DOCS-4]
requirements: [REQ-github-002]
contracts: [specs/github/github.spec.md]
limitations:
  - "Structural checks only. The release workflows, RC qualification and publication have not run: they wait for Leif's go"
observed: 2026-11-12T04:12:44Z
---
**macOS 26, branch 0xleif/release/6.3.0-prep with origin/main 9e7abb0f7 (#1067) merged in. These pass at Cargo version 6.3.0: validate-release-version.py ('Validated release version 6.3.0 across current distribution surfaces'), test-validate-release-candidate.py (OK, 67 tests) and validate-workflow-runtime-pins.py. CHANGELOG [6.3.0] now carries #1067's relayed-decision entry: workflow version 3; once a repository records one relayed decision, every machine that runs change commands there needs 6.3.0 or later; workflow-v1 relays are not gated. Diff review: the action.yml default, PUBLISHED_ACTION_DEFAULT, the docs default row, current-install guidance and the immutable @v6.2.0 prose stay at 6.2.0.**

Recorded as passed in local at revision a8bf6156b2a6, observed 2026-11-12T04:12:44Z.
