---
change: generate-and-scaffold-never-claim-files-another-spec-owns
artifact: tasks
---

# Tasks


- [x] Collect the set of source files owned by existing specs (frontmatter `files`, normalized)
- [x] Filter owned files out of `find_creation_module_sources` and `find_single_source_fallback`
- [x] Regression tests: scaffold a module whose flat file is already owned produces no spec claiming it
- [x] Update specs/generator, specs/cmd_generate (spec, requirements, 3md lesson)
