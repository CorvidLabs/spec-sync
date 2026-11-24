---
id: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-2
state: archived
type: bug_fix
base_commit: a275ab44daffac2d562df9a6fc05dc6a9a559059
---

# Advance the Action default and current guidance to published SpecSync 6.4.2

## Intent

Advance the Action default and current guidance to published SpecSync 6.4.2

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After the published v6.4.2 GitHub release (Latest, twelve assets; promoted from candidate v6.4.2-rc.2 at a275ab44, qualified on Ubuntu and macos-15 in run 45472594205), the main Action wrapper's omitted-input default moves to 6.4.2: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.4.2 and RC refs stay at a275ab44 with embedded default 6.4.1, and the docs say to pass version '6.4.2' with @v6.4.2; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary and manual-download text) names published 6.4.2; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.4.2.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. crates.io already serves 6.4.2; the floating v6 tag moves separately, after this wrapper commit's consumer checks.

## No-spec Rationale

Not applicable
