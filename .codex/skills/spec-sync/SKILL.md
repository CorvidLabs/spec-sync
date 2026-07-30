---
name: spec-sync
description: Keep markdown module specs in specs/<module>/ synchronized with source code using spec-sync. Use this whenever creating, editing, or reviewing code in a module that has (or should have) a spec, or whenever the user mentions specs, spec-sync, companion files (tasks.md/requirements.md/context.md/testing.md/design.md), or asks to add/update a module's documentation.
---

# Spec-Sync Workflow

This project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation. Specs live in `specs/<module>/<module>.spec.md`.

## Companion files

## Verified change workflow

For every meaningful source, test, public documentation, schema, or configuration change:

1. Run `specsync change new "<intent>" --json` and conduct the returned interview with the user.
2. Use `specsync change answer <id> <question-id> <answer> --json` until no questions remain.
3. Complete the adaptively selected artifacts and semantic deltas. Requirements use stable
   `REQ-<module>-<number>` IDs, a normative SHALL statement, and acceptance criteria.
4. Ask the user for the one scope approval, then run `specsync change approve <id>`.
5. Implement code, canonical specs, and tests; keep the selected artifacts current.
6. Run `specsync change check <id>`. Add global `--strict` only when requested or required by
   project policy/release/security classification; it adds validators to this same evidence path.
7. Open or update the PR and wait for the independent `SpecSync scoped review` check. Do not
   self-record an independent review.
8. After ordinary PR review and all implementation checks pass, run
   `specsync change finalize <id>`. Commit the resulting metadata/archive-only change in the same
   PR. GitHub—not SpecSync—performs the merge.

Never invent or self-grant the human scope approval or independent review. If an approved
definition or reviewed implementation changes, the corresponding digest becomes stale and must
be refreshed. `specsync change status <id>` always reports exactly one next action. Historical
repair commands remain available for older two-approval evidence but are not part of the normal
workflow.

Each canonical spec may have policy-selected companion files. Read and update the ones present; do not create empty companions only for ceremony:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** — Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* — Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read whichever companion files are present (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, `design.md`, or project-defined files)
3. After changes, run `specsync check` to verify specs still pass

## After completing work

1. Mark completed items in `tasks.md` — check off finished tasks, add new ones discovered
2. Update `context.md` — record decisions made, update current status
3. If requirements changed, update `requirements.md` acceptance criteria
4. If test coverage changed, update `testing.md` with new test files or edge cases
5. If UI/layout changed, update `design.md` with revised layout, components, or tokens

## Before creating a PR

Run `specsync change check <id>` for the active change and ensure the PR's required checks pass.
The release workflow performs the final strict/full-suite validation.

## When adding new modules

Run `specsync scaffold <module-name>` to create a spec, companion files, a registry
entry, and auto-detected source files — or `specsync new <module-name>` for a
minimal spec-only draft. Complete the spec before writing code. The
`/specsync:create-spec` command (or tool-equivalent) runs this for you, and
accepts either a bare module name or a natural-language feature description
(e.g. `/specsync:create-spec "I want a feature that lets users export their
data as CSV"`) — pass a description and it will pick a module name and use
the description to draft the spec's Purpose and Requirements.

## Key commands

- `specsync check` — validate all specs against source code
- `specsync check --json` — machine-readable validation output
- `specsync change status [id]` — show current gates and one explicit next action
- `specsync change check <id>` — materialize approved deltas and run affected-component verification
- `specsync change finalize <id>` — create the same-PR archive-only finalization; never merges externally
- `specsync coverage` — show which modules lack specs
- `specsync score` — quality score for each spec (0-100)
- `specsync scaffold <name>` — full scaffold: spec + companions + registry entry + source detection
- `specsync new <name>` — quick-create a minimal spec (add `--full` for companions)
- `specsync resolve --remote` — verify cross-project dependencies
