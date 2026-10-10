---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: tasks
---

# Tasks

- [x] Extract four private parser owners and topical tests with all files below 1000 lines.
- [x] Preserve complete function/method bodies, literals, declarations, API visibility and all 69 tests.
- [x] Synchronize canonical file coverage and parser companions.
- [x] Obtain independent preservation and security-boundary review.
- [x] Run focused parser tests, compiled inventory, formatting, production clippy, debug/release checks, release build, strict coverage and mandatory pre-push gate.

## Remaining lifecycle gates

Scoped materialization has passed. The local checkpoint is produced by the real `change check --commit` flow. Human implementation review, required publication/CI, acceptance and archival remain pending. This implementation checklist does not claim those gates are complete. Combined full-suite verification is coordinated by the parent task against the existing pristine-main baseline.
