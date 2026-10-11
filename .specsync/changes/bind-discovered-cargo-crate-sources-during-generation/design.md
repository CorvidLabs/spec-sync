---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: design
---

# Design

CoverageReport carries manifest module source paths observed through the retained coverage capability. Generation resolves configured mappings and conventional name-based paths first, then the recorded manifest paths, intersecting every binding with measured uncovered files. More specific named modules keep their files when a manifest crate spans their directories. All generation entry points use the shared coverage binding; existing destinations remain untouched.
