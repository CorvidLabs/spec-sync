---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: context
---

# Context

Main 8754ae1 includes the initial Cargo binding fix (#814). Late review still reproduces [lib].path="src/custom.rs" binding to custom rather than demo; named generation fails. A Node package named @scope/demo exits 0 without a spec because its full name is repeated as a filename beneath its module directory. Published 6.1.0 is immutable.

This repairs existing discovery and scaffold requirements. Preserve exact-target precedence, explicit mappings, existing ownership and retained-root confinement.
