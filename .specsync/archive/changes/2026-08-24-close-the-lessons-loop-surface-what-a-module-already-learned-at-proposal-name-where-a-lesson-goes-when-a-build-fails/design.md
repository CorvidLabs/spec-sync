---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: design
---

# Design

## Three surfaces, one rule

SpecSync assembles material and names the next step. It never writes a lesson and never blocks on
one. Every surface below is a pointer.

| moment | surface | silent when |
|---|---|---|
| proposal | `change new` lists module context files with line counts | no module has substantive prose |
| build | failed `change check` names the change's own `context.md` | the check passed |
| archive | `finalize` writes `lesson-bundle.md`, `next_action` names the fold | no affected specs |

## Layering

Policy in `src/change.rs`:

- `module_context_path(module)` — one definition of the convention, so surfacing and folding can
  never disagree about which file they mean
- `accumulated_lessons(root, modules)` — what counts as a lesson
- `lesson_fold_targets(root, id)` — where this change's lessons go
- `write_lesson_bundle(root, record, archive)` — assembly, best-effort

`src/commands/change.rs` renders these and decides nothing. This follows the thin-dispatch rule
already stated in `specs/cmd_change/context.md`, which the first draft of this change broke.

## Failure posture

Every path fails open. An unreadable context file yields no entry rather than an error; an
unwritable bundle leaves a successful archive intact. The alternative — a lifecycle command
failing over an authoring affordance — trades a real guarantee for a nicety.

This is deliberately the opposite posture from evidence validation, which fails closed. The
distinction is whether the artifact is *load-bearing for trust* (fail closed) or *an aid to the
author* (fail open). Lessons are the latter.

## Frontmatter

Frontmatter ends at its **closing** delimiter, never at the next `---` in the document. `---` is a
legal Markdown horizontal rule, so `split("---").nth(2)` truncates any body containing one — and
truncated material in a lesson bundle is indistinguishable from material nobody wrote. The helper
matches `view::strip_frontmatter` byte-for-byte, including BOM handling, so #696 can unify them
without a behaviour change.
