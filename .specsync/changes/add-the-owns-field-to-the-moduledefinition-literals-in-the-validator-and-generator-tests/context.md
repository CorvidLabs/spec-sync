---
change: add-the-owns-field-to-the-moduledefinition-literals-in-the-validator-and-generator-tests
artifact: context
---

# Context

`ModuleDefinition` gains `owns` (the feature change `let-a-module-own-paths-beyond-its-spec-files-…`). Three struct literals in the `validator` and `generator` unit tests spell every field of `ModuleDefinition` out, so they stop compiling until the field is added. That is the whole of this change: `owns: Vec::new()` / `owns: vec![]` in test fixtures, in files whose specs do not change. It is declared as a `--no-spec-change` workspace rather than by inventing `validator` and `generator` deltas, because nothing about those modules' contracts moved.
