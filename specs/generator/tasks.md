---
spec: generator.spec.md
---

## Tasks

(none open for 5.0)

## Done

- [x] Bind manifest-discovered Cargo sources and preserve existing/spec-specific module owners; cover CLI modes and library entry points (#813).

- [x] Deterministic built-in and custom template generation
- [x] Companion files and safe no-overwrite behavior
- [x] Remove provider parameters and AI error outcomes
- [x] Bind CLI generation reads and writes to one retained project-root capability

- [x] Correct same-name crate and exact Cargo target bindings from PR #814 review.
- [x] Preserve missing explicit mappings and assign shared or overlapping manifest sources once; retain narrower targets during batch generation (#814 additional review).

- [x] Prepare the 6.1.1 discovery regression fixes and distribution contracts (fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1).

- [x] Implement the approved exact-file creation selection correction with confined no-follow metadata traversal, explicit mapping precedence and causal focused regressions. Full final-source and independent release qualification remain separate.
- [x] Create spec and companion files through `util::create_project_file` (REQ-generator-010). Evidence: `companion_files_are_never_created_through_a_link_that_leaves_the_project`, `scaffold_refuses_a_dangling_companion_link_and_creates_the_others`.
