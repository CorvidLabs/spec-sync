---
id: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
state: archived
type: bug_fix
base_commit: d08d2746f635bc26900601376d7392395d07faa9
---

# Advance the published SpecSync 6.1.2 Action default and archive-gate examples after platform consumer checks

## Intent

Advance the published SpecSync 6.1.2 Action default and archive-gate examples after platform consumer checks

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After the published v6.1.2 GitHub release (10 assets, Latest; candidate v6.1.2-rc.2 at d08d2746 qualified on Ubuntu and macos-15) and crates.io 6.1.2, advance the main Action wrapper's omitted-input default to 6.1.2: action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in site/src/content/docs/integrations/github-action.md. Composite runtime script bytes and the explicit CI and Trust candidate mirrors are unchanged.
- Archive-gate examples (require-archived, archive-gate-only) in README and site docs move from @main to @v6.1.2 with version '6.1.2'; the validator no longer allows @main pins for them, and README and CHANGELOG stop describing 6.1.2 as unpublished.
- Immutable v6.1.2 and RC refs stay at qualified candidate d08d2746 with embedded default 6.1.1; docs say to pass version '6.1.2' with @v6.1.2. The immutable v6.0.0 caveat is preserved. The github spec Purpose is updated through one normal SpecSection delta; permanent requirements and history rows are preserved.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. Floating v6 moves only after this change is merged and omitted-input Linux and macOS Action consumers pass at the merged wrapper commit, with no immutable tag movement.

## No-spec Rationale

Not applicable
