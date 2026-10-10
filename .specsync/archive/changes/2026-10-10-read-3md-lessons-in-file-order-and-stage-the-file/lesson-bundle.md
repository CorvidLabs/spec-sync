# Lesson bundle — read-3md-lessons-in-file-order-and-stage-the-file

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Read 3md lessons in file order and stage the file
- **Kind**: BugFix
- **Specs**: change
- **Paths**: Cargo.toml, specs/change/change.spec.md, specs/change/context.md, specs/change/requirements.md, specs/cmd_init/cmd_init.spec.md, src/change.rs, src/change_tests.rs
- **Acceptance**: The first plane in the file is context, the lesson bundle names the 3md file, and change check commits a new module.3md.

## Evidence

- Verification commit: `24a27dcec999e443dd52b944aefd20e84281e5c5`
- Base commit: `8d6fcc2fbfc792b2e515935c9ef6ce00c75724a2`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

The first plane in the file is context, whatever z it carries. Later planes are lessons. The lesson bundle names that file. change check stages a new module.3md. An unreadable 3md file falls back to context.md.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md` passes with a higher-z context plane first, names the 3md file in the lesson bundle, and falls back to context.md when the 3md file does not parse. |

- `cargo test --bin specsync -- a_module_3md_file_shows_later_planes` passed.

## Where these lessons go

- `specs/change/context.md`
