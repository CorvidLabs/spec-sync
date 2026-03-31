# Agent Instructions — spec-sync

This project uses **spec-sync** to keep module specs (`*.spec.md`) aligned with source code.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `specsync check --strict` | Validate specs against code — fix stale, phantom, or missing entries |
| `specsync coverage` | Find modules with no spec coverage |
| `specsync generate --provider auto` | Create specs for uncovered modules |
| `specsync score` | Score spec quality — target ≥ 80 per spec |
| `specsync hooks install` | Install git pre-commit hooks and IDE agent snippets |
| `specsync resolve --remote` | Resolve cross-project spec references |

## Workflow

1. **Before writing or updating specs**, read the source files first. Never invent exports.
2. Run `specsync check --strict` and fix all errors before committing.
3. Run `specsync score` and improve any spec scoring below 80.
4. Increment the spec `version` field whenever you change it.

## Spec Format

Each `*.spec.md` needs YAML frontmatter (`module`, `version`, `status`, `files`) and sections: Purpose, Requirements, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log. Public API tables must use backtick-quoted names matching actual code exports.

## MCP Integration

For richer integration, run `specsync mcp` to start the MCP server. This exposes `specsync_check`, `specsync_generate`, `specsync_coverage`, and `specsync_score` as callable tools.
