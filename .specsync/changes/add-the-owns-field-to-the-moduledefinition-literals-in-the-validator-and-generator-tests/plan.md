---
change: add-the-owns-field-to-the-moduledefinition-literals-in-the-validator-and-generator-tests
artifact: plan
---

# Plan

1. Add `owns: Vec::new()` to the `ModuleDefinition` literal in `src/validator.rs` (`compute_coverage_checked` module-mapping test) and `owns: vec![]` to the two literals in `src/generator.rs` (`find_files_for_module` tests).
2. `cargo test validator generator` — the affected tests pass unchanged; `fledge run lint` stays green.
