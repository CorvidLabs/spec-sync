---
id: advance-the-action-default-and-current-guidance-to-published-specsync-6-3-0
state: archived
type: bug_fix
base_commit: b87bbd88efea06ffd9474af02018240f24300419
---

# Advance the Action default and current guidance to published SpecSync 6.3.0

## Intent

Advance the Action default and current guidance to published SpecSync 6.3.0

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After the published v6.3.0 GitHub release (Latest, twelve assets; promoted from candidate v6.3.0-rc.1 at b87bbd88, qualified on Ubuntu and macos-15 in run 43162493255), the main Action wrapper's omitted-input default moves to 6.3.0: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.3.0 and RC refs stay at b87bbd88 with embedded default 6.2.0, and the docs say to pass version '6.3.0' with @v6.3.0; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary, comment-fallback and manual-download text) names published 6.3.0; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.3.0.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. The PR merges only after crates.io serves 6.3.0; floating v6 moves only on Leif's separate decision.

## No-spec Rationale

Not applicable
