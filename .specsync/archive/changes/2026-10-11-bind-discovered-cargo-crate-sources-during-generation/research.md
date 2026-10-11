---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: research
---

# Research

The same naming-only binding exists in 6.0.1. Cargo manifest discovery maps package names to project-relative src roots and workspace names to member roots. The retained generator intersects mappings with uncovered files. A global single-file fallback would claim unrelated files for missing module names. Root replacement and degraded-manifest behavior must retain their existing contracts.
