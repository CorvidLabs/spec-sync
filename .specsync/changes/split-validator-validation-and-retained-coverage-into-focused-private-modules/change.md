---
id: split-validator-validation-and-retained-coverage-into-focused-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split validator validation and retained coverage into focused private modules

## Intent

Split validator validation and retained coverage into focused private modules

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- Organize validator implementation and existing unit tests into cohesive private modules with the facade and every new source or test file below 1000 lines where a coherent boundary permits; preserve public and crate-visible API paths, validation ordering, severity, diagnostic text, coverage JSON and metrics, retained capabilities, identity and race checks, deterministic traversal and shared budgets; retain all existing tests and platform guards; synchronize canonical spec coverage and verify behavior against the unchanged baseline without weakening security or lifecycle gates

## No-spec Rationale

Not applicable
