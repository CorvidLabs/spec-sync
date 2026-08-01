# Agent Instructions — spec-sync

> **Documentation site:** The project documentation lives at `site/` (Astro + MDX). Run `cd site && bun install && bun run build` to build locally. Content is in `site/src/content/docs/`. Do **not** modify `docs/src/` — that mdBook tree has been removed.

This project uses **spec-sync** to keep module specs (`*.spec.md`) aligned with source code.
Enforcement is **strict** — CI and pre-commit hooks will block on any spec violation.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `specsync check --strict` | Validate specs against code — fix stale, phantom, or missing entries |
| `specsync check --fix` | Auto-add undocumented exports to spec Public API tables |
| `specsync coverage` | Find modules with no spec coverage |
| `specsync generate` | Deterministically create specs for uncovered modules |
| `specsync score` | Score spec quality — target ≥ 80 per spec |
| `specsync new <name>` | Quick-create a minimal spec with auto-detected source files |
| `specsync scaffold <name>` | Full scaffold: spec + companions + registry entry + source detection |
| `specsync add-spec <name>` | Scaffold a spec with companion files (tasks.md, context.md) |
| `specsync hooks install` | Install git pre-commit hooks and IDE agent snippets |
| `specsync agents install` | Install skills plus `/specsync:create-spec`, `create-change`, `check`, and `audit` (Claude/Cursor/Gemini) |
| `specsync resolve --remote` | Resolve cross-project spec references |
| `specsync diff --base <ref>` | Show export changes since a git ref (useful for CI/PR reviews) |
| `specsync report` | Per-module coverage report with stale/incomplete detection |
| `specsync comment --pr N` | Post spec-check summary as a PR comment |
| `specsync changelog <range>` | Generate changelog of spec changes between git refs |
| `specsync deps` | Validate cross-module dependency graph (`--mermaid`, `--dot`) |
| `specsync compact` | Compact changelog tables by summarizing old entries |
| `specsync archive-tasks` | Move completed task items to archive section |
| `specsync merge` | Auto-resolve git merge conflicts in spec files |
| `specsync change new <desc>` | Create a draft SDD change with the deterministic interview |
| `specsync change approve/check/finalize <id>` | Drive the single workflow: one scope approval, targeted verification, scoped PR review, and same-PR finalization |
| `specsync change status [id]` | Show current gates and exactly one explicit next action |
| `specsync change reopen <id>` | Re-verify stale accepted evidence (audited, append-only) |
| `specsync change correct-owner <id>` | Append audited exact owner corrections (single `--path/--spec`, or batch: repeated flags, `--manifest`, `--all-missing`) |
| `specsync change finalize <id>` | Validate current review/evidence and move the package into the dated archive in the same PR; GitHub performs the merge |
| `specsync change ship-status` | Report HEAD tip class + staged product→review→archive ship sequence |
| `specsync change check [id]` | Scoped verification for one change (materialize + targeted tests); not archive history |
| `specsync change audit` | Project health over active workspaces and living specs (archives are history) |
| `specsync migrate 5.0` | Backfill 5.0.1-era reopening digest fields idempotently (the remediation `check` prints for missing-field ledgers) |


## Before pushing (MANDATORY — keep it FAST)

**Never `git push` without a green pre-push gate.** The CI failures that look "obvious" (fmt, path coverage / SDD active change, undocumented exports) are exactly what this gate catches locally.

```bash
fledge lanes run pre-push
# or: ./scripts/pre-push-gate.sh
```

**Target: seconds to ~2 minutes on a warm tree.** This is intentionally *not* a full test suite.

| Step | What | Why fast |
|------|------|----------|
| 1 | `cargo fmt --check` | seconds |
| 2 | `cargo check` | incremental types only (no clippy, no tests) |
| 3 | `specsync check --strict --require-coverage 100 --force` | uses `target/release/specsync` when present |

**Do not** put these in pre-push (they belong in `fledge lanes run verify` / CI):

- full `cargo test` / `change check` verification suite
- `cargo clippy -D warnings` (run in verify)
- docs site / vscode package builds

If step 3 fails with *meaningful changed paths are not covered by an active change*:

1. Create or update an active SDD change covering every dirty path (`change new` / expand `affected_paths`)
2. Fill artifacts completely (no HTML TODO comment stubs)
3. `change approve` then ensure the change is **implementing** or **verifying** (approved-only does not cover paths)
4. Document any new public exports in the module spec Public API table
5. Re-run `fledge lanes run pre-push`

When the PR is merge-ready (not every push):

```bash
fledge lanes run verify   # clippy + full test + release build + spec-check
```

Do **not** rely on remote CI as the first formatter or path-coverage check.


## Spec Lifecycle

Specs follow a lifecycle from creation through archival:

1. **Requirements** — Create `requirements.md` in the spec directory. These are high-level acceptance criteria and user stories. They are permanent invariants, not tasks.
2. **Spec creation** — Run `specsync scaffold <name>` or `specsync new <name>` to create the spec, companion files, and detect source files. Complete the spec before writing code.
3. **Active development** — The spec (`*.spec.md`) is the detailed contract. Keep it in sync with code changes. Use `tasks.md` for work items, `context.md` for architectural decisions.
4. **Working specs** — Specs with `status: draft` are in-progress. Promote to `status: stable` once the module's API is settled.
5. **Maintenance** — Run `specsync check --strict` to catch drift. Use `specsync compact` to keep changelogs manageable.
6. **Archival** — When a module is deprecated, set `status: deprecated`. Use `specsync archive-tasks` to clean up completed work items.

## Companion Files

Each spec in `specs/<module>/` has companion files — read them before working, update them after:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.

## Workflow

### Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read companion files: `tasks.md`, `requirements.md`, and `context.md`
3. Understand the existing API contract before making changes

### After making changes

1. Update the spec's Public API table if exports changed
2. Increment the spec `version` field
3. Add a Change Log entry with the date and description
4. Mark completed items in `tasks.md`, add new ones discovered
5. Update `context.md` with decisions made and current status
6. If requirements changed, update `requirements.md`
7. Run `specsync check --strict` and fix all errors

### Before creating a PR

1. Run `specsync check --strict` — all specs must pass with zero warnings
2. Run `specsync score` and improve any spec scoring below 80
3. CI will **fail** if specs are out of sync (enforcement is strict)

## Spec Format

Each `*.spec.md` needs YAML frontmatter (`module`, `version`, `status`, `files`) and sections: Purpose, Requirements, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log. Public API tables must use backtick-quoted names matching actual code exports.

## MCP Integration

For richer integration, run `specsync mcp` to start the MCP server. This exposes `specsync_check`, `specsync_generate`, `specsync_coverage`, and `specsync_score` as callable tools.

<!-- CorvidLabs trust toolchain: BEGIN (managed, do not edit inside) -->
## CorvidLabs trust toolchain

This repository uses one trust gate. Every session must use it and must not bypass or weaken it.

- Run `fledge trust verify` before calling a change complete.
- Keep module specs synchronized with implementation changes.
- Treat an Augur block verdict as a hard stop that must be surfaced and de-risked.
- Record and verify provenance with Attest after the repository's verification lane passes.
- Keep generated trust configuration and this managed block in place.

<!-- CorvidLabs trust toolchain: END -->
