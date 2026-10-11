---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: context
---

# Context

Issue #813 reproduces on released 6.1.0: init finds src, coverage names the Cargo package demo and src/lib.rs, but generate matches only directory/file names and skips demo. The user explicitly requested fixing the issue. Coverage already has checked manifest mappings; preserve them rather than reopening Cargo.toml during generation.
