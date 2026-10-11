---
spec: generator.spec.md
---

## User Stories

- As a developer, I want reproducible local scaffolds for every uncovered module.
- As a team, I want custom templates and existing files preserved.
- As a coding-agent user, I want standard companion files ready for enrichment.

## Constraints

- Generated files must be valid inputs to `specsync check`.
- Source discovery excludes tests and remains deterministic.

### REQ-generator-001

The `generator` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-generator-002

The CLI generator SHALL keep every project filesystem side effect beneath one retained
project-root capability.

Acceptance Criteria

- Template reads, module-directory creation, spec publication, and companion publication are
  relative to the retained capability.
- Configured specs paths and module destinations reject absolute, rooted, prefix, and parent
  traversal components before use.
- Redirecting the caller-visible root after checked coverage cannot redirect an output write.
- Existing files remain no-overwrite destinations.


### REQ-generator-003

Generated output SHALL NOT embed a coverage percentage that was not measured.

Acceptance Criteria
- A zero denominator renders the unmeasured state.

### REQ-generator-004

Generated specs SHALL contain only symbols the configured surface includes.

Acceptance Criteria
- A generated spec, once activated, passes `check` without orphan-export errors.
- The tool cannot emit a spec its own validator rejects.

### REQ-generator-005

Generation SHALL record unspecced modules skipped because no source files were found, and SHALL confine generated destinations beneath the retained project root.

Acceptance Criteria
- `GenerationOutcome.skipped_no_files` is populated on the retained path and both ambient generate paths when a module has no files.
- `confined_generation_path` is `pub(crate)` so `scaffold --dir` can reuse it.
- Absolute paths and `..` components are refused with `must remain beneath the retained project root`.

### REQ-generator-006

The generator SHALL bind a manifest-discovered Cargo module to its measured uncovered source files when conventional module-name paths do not resolve, preserving explicit configuration, existing spec ownership, and more specific named module bindings.

Acceptance Criteria

- The issue #813 init then generate fixture creates specs/demo/demo.spec.md with src/lib.rs and add.
- Text, JSON, batch, and library generation entry points use the same binding.
- Library/binary roots and workspace members stay scoped to their declared sources.
- Files already referenced by specs and files belonging to other named modules are not claimed by the fallback.
- Missing configured modules still report skipped_no_files.

