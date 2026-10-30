---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: requirements
---

# Requirements

### REQUIREMENT REQ-generator-006

The generator SHALL bind a manifest-discovered Cargo module to its measured uncovered source files when conventional module-name paths do not resolve, preserving explicit configuration, existing spec ownership, and more specific named module bindings.

Acceptance Criteria

- The issue #813 init then generate fixture creates specs/demo/demo.spec.md with src/lib.rs and add.
- Text, JSON, batch, and library generation entry points use the same binding.
- Library/binary roots and workspace members stay scoped to their declared sources.
- Files already referenced by specs and files belonging to other named modules are not claimed by the fallback.
- Missing configured modules still report skipped_no_files.

