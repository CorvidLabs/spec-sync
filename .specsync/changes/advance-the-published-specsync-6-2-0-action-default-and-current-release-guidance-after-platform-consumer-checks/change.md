---
id: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
state: implementing
type: bug_fix
base_commit: 40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56
---

# Advance the published SpecSync 6.2.0 Action default and current release guidance after platform consumer checks

## Intent

Advance the published SpecSync 6.2.0 Action default and current release guidance after platform consumer checks

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After the published v6.2.0 GitHub release (Latest, ten assets; promoted from candidate v6.2.0-rc.2 at 40dcf33a, qualified on Ubuntu and macos-15 in run 42035620884) and crates.io 6.2.0, advance the main Action wrapper's omitted-input default to 6.2.0: action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in site/src/content/docs/integrations/github-action.md. Composite runtime script bytes and the explicit CI and Trust candidate mirrors are unchanged.
- Immutable v6.2.0 and RC refs stay at qualified candidate 40dcf33a with embedded default 6.1.2; docs say to pass version '6.2.0' with @v6.2.0. The immutable v6.0.0 caveat is preserved. The github spec Purpose is updated through one normal SpecSection delta; permanent requirements and history rows are preserved.
- Current guidance stops describing 6.2.0 as unpublished or pinning 6.1.2: README, SECURITY.md, MIGRATION.md current pins, docs/ADOPTING.md, the site index, quickstart, ci-gate and polyglot candidate notes, the github-action.md preparation paragraph and manual-CI download, and the cli and workflow reconcile-history availability wording. CHANGELOG dates 6.2.0 and gains an Unreleased entry; docs/releases/6.2.0.md records publication. Historical release text is unchanged.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. Floating v6 moves only after this change is merged and omitted-input Linux and macOS Action consumers pass at the merged wrapper commit, with no immutable tag movement.

## No-spec Rationale

Not applicable
