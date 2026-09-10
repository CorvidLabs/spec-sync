---
id: correct-remaining-specsync-6-0-docs-after-the-stable-ship
state: implementing
type: documentation
base_commit: 6f11e34ef65710dd8839bfaaa6e6bc9004d6a530
---

# Correct remaining SpecSync 6.0 docs after the stable ship

## Intent

Correct remaining SpecSync 6.0 docs after the stable ship

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- docs/RELEASING.md records that promote executed for v6.0.0 on 2026-09-09 and uses a next-release example instead of a second 6.0.0 promotion
- docs/ci-confidence.md records that promote executed and does not claim the final tag is missing
- site polyglot example does not teach specsync check --explain for language detection or invented language_overrides
- site CI-gate example dual-pins CorvidLabs/spec-sync@v6.0.0 with version 6.0.0 and uses real Action inputs
- github spec companions do not require Windows for 6.0 RC qualification

## No-spec Rationale

Not applicable
