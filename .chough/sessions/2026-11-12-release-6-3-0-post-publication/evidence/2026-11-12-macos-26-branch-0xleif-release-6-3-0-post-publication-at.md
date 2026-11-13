---
kind: evidence
id: 2026-11-12-macos-26-branch-0xleif-release-6-3-0-post-publication-at
aliases: [OBS-structural-17f25f9e2566d0f32aae5a6b]
claim: "macOS 26, branch 0xleif/release/6.3.0-post-publication at e1867319a (Leif's definition approval committed on top of 4ef501b84). These pass: validate-release-version.py ('Validated release version 6.3.0 across current distribution surfaces', PUBLISHED_ACTION_DEFAULT 6.3.0), validate-workflow-runtime-pins.py, test-validate-release-candidate.py (OK, 67 tests), action.yml parses with version default 6.3.0, and the pre-push gate (fmt, cargo check, strict spec coverage 62/62 specs, 100% files) passes. Since b87bbd88 the only Action-surface diff is action.yml's version default and description; no composite runtime script changed. rg over the touched docs leaves only historical 6.2.0 mentions and the immutable @v6.3.0-embeds-6.2.0 notes."
result: passed
evidence_kind: structural
method: agent-assisted
revision: e1867319a2255cfa6327e11dc9378b7975df21db
content: 733387c76ecc181ba8d2834bf81af65a30a41d1cf29969ad8cdd1139b10a5e5f
environment: local
criteria: [CI-1.a, DOCS-4]
requirements: [REQ-github-002]
contracts: [specs/github/github.spec.md]
limitations:
  - Full test suite not run locally; hosted CI runs it on the archived tip
  - crates.io still serves 6.2.0; the install guidance assumes 6.3.0 there, so the PR waits for cargo publish
observed: 2026-11-12T14:20:26Z
---
**macOS 26, branch 0xleif/release/6.3.0-post-publication at e1867319a (Leif's definition approval committed on top of 4ef501b84). These pass: validate-release-version.py ('Validated release version 6.3.0 across current distribution surfaces', PUBLISHED_ACTION_DEFAULT 6.3.0), validate-workflow-runtime-pins.py, test-validate-release-candidate.py (OK, 67 tests), action.yml parses with version default 6.3.0, and the pre-push gate (fmt, cargo check, strict spec coverage 62/62 specs, 100% files) passes. Since b87bbd88 the only Action-surface diff is action.yml's version default and description; no composite runtime script changed. rg over the touched docs leaves only historical 6.2.0 mentions and the immutable @v6.3.0-embeds-6.2.0 notes.**

Recorded as passed in local at revision e1867319a225, observed 2026-11-12T14:20:26Z.
