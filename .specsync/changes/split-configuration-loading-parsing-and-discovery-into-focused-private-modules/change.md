---
id: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split configuration loading parsing and discovery into focused private modules

## Intent

Split configuration loading parsing and discovery into focused private modules

## Affected Canonical Specs

- `config`

## Acceptance Criteria

- Split the configuration implementation and all existing tests into cohesive private owners with the facade and each new file below 1000 lines; preserve public and crate-visible API paths and complete bodies, literals, attributes and cfg gates; preserve exact-byte retained parsing without pathname reopening, supplied-source authority without ambient rediscovery, strict-loader refusal, GitHub invalid sentinel behavior, value-safe retired-key diagnostics and lossless migration; retain every test and complete canonical spec coverage without weakening trust or lifecycle gates

## No-spec Rationale

Not applicable
