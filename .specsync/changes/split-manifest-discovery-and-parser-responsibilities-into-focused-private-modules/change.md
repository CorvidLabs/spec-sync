---
id: split-manifest-discovery-and-parser-responsibilities-into-focused-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split manifest discovery and parser responsibilities into focused private modules

## Intent

Split manifest discovery and parser responsibilities into focused private modules

## Affected Canonical Specs

- `manifest`

## Acceptance Criteria

- Organize manifest discovery implementation and existing tests into cohesive private modules with the facade and every new source or test file below 1000 lines wherever coherent boundaries permit; preserve public and crate-visible API paths, fixed parser order and first-name-wins merging, all literal grammar and diagnostic behavior, retained capability and identity checks, workspace charging and memoization, limits and platform guards; retain every existing regression and complete canonical spec coverage without weakening trust or lifecycle gates

## No-spec Rationale

Not applicable
