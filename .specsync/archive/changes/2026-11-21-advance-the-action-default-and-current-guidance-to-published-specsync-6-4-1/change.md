---
id: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-1
state: archived
type: bug_fix
base_commit: 3f0985e80acab0fe4559fe322cf929be40057414
---

# Advance the Action default and current guidance to published SpecSync 6.4.1

## Intent

Advance the Action default and current guidance to published SpecSync 6.4.1

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After the published v6.4.1 GitHub release (Latest, twelve assets; promoted from candidate v6.4.1-rc.1 at 3f0985e8, qualified on Ubuntu and macos-15 in run 44843175474), the main Action wrapper's omitted-input default moves to 6.4.1: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.4.1 and RC refs stay at 3f0985e8 with embedded default 6.4.0, and the docs say to pass version '6.4.1' with @v6.4.1; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary and manual-download text) names published 6.4.1; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.4.1.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. The PR merges only after crates.io serves 6.4.1; the floating v6 tag moves separately, after this wrapper commit's consumer checks.

## No-spec Rationale

Not applicable
