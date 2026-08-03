---
change: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
artifact: tasks
---

# Tasks

- [x] Resolve evidence completeness before running verification commands
- [x] Name the artifact and section in the evidence-gap message
- [x] Name the failing command and exit code in the command-failure message
- [x] Converge on an `## ADDED` block already present with identical content
- [x] Keep present-but-different an error that directs the author to `## MODIFIED`
- [x] Reject duplicate ordinals at definition approval and in `change audit`
- [x] Add focused regression tests for delta convergence and ordinal identity
- [x] Run the full Rust suite
