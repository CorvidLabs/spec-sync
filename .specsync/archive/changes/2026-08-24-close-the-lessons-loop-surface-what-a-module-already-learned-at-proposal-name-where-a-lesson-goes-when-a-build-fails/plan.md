---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: plan
---

# Plan

1. Domain policy in `src/change.rs`: `module_context_path`, `accumulated_lessons`,
   `lesson_fold_targets`, `strip_frontmatter`, `write_lesson_bundle`.
2. Command layer in `src/commands/change.rs` reduced to rendering at all three surfaces.
3. `finalize_change` assembles the bundle after a successful archive, best-effort.
4. Regression tests for the stripper and the substantive-prose count.
5. Spec updates to `specs/change/change.spec.md` and `specs/cmd_change/cmd_change.spec.md`.
