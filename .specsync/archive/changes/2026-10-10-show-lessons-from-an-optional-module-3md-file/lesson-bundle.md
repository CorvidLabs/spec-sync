# Lesson bundle — show-lessons-from-an-optional-module-3md-file

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Show lessons from an optional module 3md file
- **Kind**: Feature
- **Specs**: change
- **Paths**: Cargo.toml, Cargo.lock, hi/three.md, specs/change/change.spec.md, specs/change/requirements.md, src/change.rs, src/change_tests.rs
- **Acceptance**: When a module has <module>.3md, the next change shows the lessons on the later planes, and a module without that file still uses context.md.

## Evidence

- Verification commit: `8bfe10f6c63108539435a845a3e367f16de07ecf`
- Base commit: `6891ac2ed97a98bfa24c5e66a8383a482948b0df`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Lessons and context can live in specs/<module>/<module>.3md. The spec stays module.spec.md. context.md remains the lesson source until that 3md file exists.

## From the change's design.md

# Design

When specs/<module>/<module>.3md exists and parses, the briefing reads every plane after the lowest one and names that file. The lowest plane is context and is not quoted as a lesson. When the file is absent, context.md is unchanged as the lesson source. A bad file yields no lesson and does not fail the command.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md` passes, and the existing context.md briefing tests still pass. |

- `cargo test --bin specsync -- a_module_3md_file_shows_later_planes` passed.

## Where these lessons go

- `specs/change/context.md`
