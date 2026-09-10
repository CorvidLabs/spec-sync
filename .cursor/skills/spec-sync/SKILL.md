---
name: spec-sync
description: Keep markdown module specs in specs/<module>/ synchronized with source code using specsync check (the product). Use this whenever creating, editing, or reviewing a spec or its source, or when the user mentions spec-sync, SDD, change adopt, companion files, or the 6.0 change lifecycle (approve, check --commit, review, ship).
---

# Spec-Sync Workflow

This project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation. Specs live in `specs/<module>/<module>.spec.md`.

`specsync check` is the product. Bidirectional spec-to-code validation does not require an
active change. SDD (`specsync change`) is opt-in: `specsync init` writes it off;
`specsync change adopt` turns it on.

## Verified change lifecycle (6.0)

Happy path verbs: `change new` -> `answer` -> `approve --actor` -> implement ->
`check --commit` -> `review --reviewer` -> `ship`/`finalize`. `start` / `verify` /
`accept` / `archive` are v1 recovery, not the happy path. Change ids are slugs, not
`CHG-NNNN`.

When SDD is adopted and the work is a meaningful source, test, public documentation,
schema, or configuration change:

1. Run `specsync change new "<intent>" --json` and conduct the returned interview with the user.
2. Use `specsync change answer <id> <question-id> "<answer>" --json` until no questions remain.
3. Complete the adaptively selected artifacts and semantic deltas. Requirements use stable
   `REQ-<module>-<number>` IDs, a normative SHALL statement, and acceptance criteria.
4. Ask the user for the single scope approval, then run `specsync change approve <id> --actor "<identity>"`.
5. Implement code, canonical specs, and tests on the same branch. Run `specsync change check [<id>] --commit`
   for **scoped** verification of this change only (materialize deltas + spec↔code sync). Do **not**
   treat check as a full archive integrity walk. Use `specsync change audit` when you need project
   health over **active** workspaces and living specs. Archives are history.
6. Complete ordinary pull-request review. For agent-authored work, have a human inspect the
   change package, implementation diff, canonical spec delta, and targeted evidence once, then
   record it with `specsync change review <id> --reviewer "<identity>"`. The reviewer MAY be the
   same person who approved the definition. Do not invent a second identity for solo work.
7. Run `specsync change ship <id>` (or `finalize`) for the same-PR archive tip. Do not commit
   between review and ship. Merge on GitHub. SpecSync does not merge the pull request.

## Lifecycle verbs

- `specsync check` - validate all specs against source. Default daily path. No change required.
- `specsync change check [id]` - verify **this** change (materialize + spec↔code sync). Add `--commit` for ship-ready product-tip evidence.
- `specsync change audit` - project health over **active** workspaces and living specs. Not archive history.
- Archives are history; do not re-validate terminal evidence for every archived change on each check.
- Slash commands after `specsync agents install`: Claude and Gemini use `/specsync:create-spec`, `/specsync:create-change`, `/specsync:check`, `/specsync:audit`. Cursor uses the flat names `/specsync-create-spec`, `/specsync-create-change`, `/specsync-check`, `/specsync-audit`.

Never invent or self-grant the scope approval. If an approved definition
changes, its digest becomes stale and must be approved again. `specsync change status` always
prints one explicit next action and one `Handoff:` line. Clear context (or hand the change to a
fresh session) only when `Handoff:` says `safe`; otherwise do what `Before clearing:` names first.
Resume with `specsync change status <id>`. Historical repair commands remain available for older
evidence, but new changes use this single workflow.

Each canonical spec may have policy-selected companion files. Read and update the ones present; do not create empty companions only for ceremony:

- **`tasks.md`** - Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** - Acceptance criteria and user stories. These are permanent invariants, not tasks: do not check them off. Update if requirements change.
- **`context.md`** - Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** - Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* - Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read whichever companion files are present (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, `design.md`, or project-defined files)
3. After changes, run `specsync check` to verify specs still pass

## After completing work

1. Mark completed items in `tasks.md`: check off finished tasks, add new ones discovered
2. Update `context.md`: record decisions made, update current status
3. If requirements changed, update `requirements.md` acceptance criteria
4. If test coverage changed, update `testing.md` with new test files or edge cases
5. If UI/layout changed, update `design.md` with revised layout, components, or tokens

## Before creating a PR

Run `specsync check --strict`: all specs must pass with zero warnings. If SDD is adopted, push
the product tip from `change check --commit` with the PR. Record `change review` and
`change ship`/`finalize` only after that tip is on the PR and required checks have run. Do not
record review or ship before the PR exists.

## When adding new modules

Run `specsync scaffold <module-name>` to create a spec, companion files, a registry
entry, and auto-detected source files, or `specsync new <module-name>` for a
minimal spec-only draft. Complete the spec before writing code. The
`/specsync:create-spec` command (or tool-equivalent) runs this for you, and
accepts either a bare module name or a natural-language feature description
(e.g. `/specsync:create-spec "I want a feature that lets users export their
data as CSV"`): pass a description and it will pick a module name and use
the description to draft the spec's Purpose and Requirements.

## Key commands

- `specsync check` - validate all specs against source code
- `specsync check --json` - machine-readable validation output
- `specsync change adopt` - turn on the opt-in verified change workflow
- `specsync change new "<intent>"` / `change answer` - open a change and finish the interview
- `specsync change approve <id> --actor "<identity>"` - record the single scope approval
- `specsync change check [id]` - scoped verification for one SDD change (`--commit` for ship evidence)
- `specsync change review <id> --reviewer "<identity>"` - record scoped implementation review
- `specsync change ship <id>` / `change finalize` - same-PR archive tip; do not commit between review and ship
- `specsync change status [id]` - one next action and a `Handoff:` line
- `specsync change audit` - active workspaces + living specs (not archive history)
- `specsync coverage` - show which modules lack specs
- `specsync score` - quality score for each spec (0-100)
- `specsync scaffold <name>` - full scaffold: spec + companions + registry entry + source detection
- `specsync new <name>` - quick-create a minimal spec (add `--full` for companions)
- `specsync resolve --remote` - verify cross-project dependencies
