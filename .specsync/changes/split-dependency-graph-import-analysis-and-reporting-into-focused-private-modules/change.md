---
id: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split dependency graph import analysis and reporting into focused private modules

## Intent

Split dependency graph import analysis and reporting into focused private modules

## Affected Canonical Specs

- `deps`

## Acceptance Criteria

- Split the dependency implementation and all 43 existing tests into cohesive private owners with the facade and each new file below 1000 lines; preserve public and crate-visible API paths, complete function bodies, literals, attributes and cfg gates; preserve import dialect authority, source-root guards and unreadable-file errors, Rust masking and ownership, JVM declaration-before-layout attribution including ambiguous and unresolved outcomes, cycles, topological order and rendering; keep canonical spec coverage complete without weakening trust or lifecycle gates

## No-spec Rationale

Not applicable
