---
id: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
state: verifying
type: bug_fix
base_commit: fffbef561fe7cbcfad669dac1855e711896deca4
---

# Preserve punctuated Public API symbols across all export extractors

## Intent

Preserve punctuated Public API symbols across all export extractors

## Affected Canonical Specs

- `parser`
- `ignore`

## Acceptance Criteria

- Public API tables preserve the complete first nonempty backtick symbol while malformed rows and excluded subsections stay ignored and a GitHub Actions YAML spec passes strict validation with all 30 Trust exports documented and every Fledge lane plus Trust dogfood passes.

## No-spec Rationale

Not applicable
