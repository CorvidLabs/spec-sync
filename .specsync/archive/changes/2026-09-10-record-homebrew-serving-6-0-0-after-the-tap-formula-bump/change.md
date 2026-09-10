---
id: record-homebrew-serving-6-0-0-after-the-tap-formula-bump
state: archived
type: documentation
base_commit: 0188edba726e738190468a90bb17070a667996e0
---

# Record Homebrew serving 6.0.0 after the tap formula bump

## Intent

Record Homebrew serving 6.0.0 after the tap formula bump

## Affected Canonical Specs

- None

## Acceptance Criteria

- Every present-tense claim that the Homebrew tap serves SpecSync 5.2.0 is gone from README.md, MIGRATION.md, docs/ADOPTING.md, docs/RELEASING.md, docs/ci-confidence.md, and site/src/content/docs/quickstart.md. Those documents now say Homebrew serves 6.0.0, which matches CorvidLabs/homebrew-tap Formula/spec-sync.rb at version 6.0.0 on main. docs/RELEASING.md additionally records that Formula/corvid-trust.rb asserts its dependency versions in its test block, so the two formulae must be bumped in the same change. Historical records under docs/6-0-*, docs/GOAL-*, docs/superpowers, specs/, and CHANGELOG.md are left untouched.

## No-spec Rationale

Documentation only. The Homebrew tap now serves 6.0.0, so the caveats saying it still serves 5.2.0 are stale. No module contract, public API, or behavior changes.
