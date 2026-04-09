---
module: cmd_comment
version: 1
status: stable
files:
  - src/commands/comment.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/comment/comment.spec.md
  - specs/github/github.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Comment

## Purpose

Implements the `specsync comment` command. Generates a spec-sync check summary as markdown and optionally posts it as a GitHub PR comment via `gh pr comment`.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_comment` | `root: &Path, pr: Option<u64>, _base: &str` | `()` | Generate check summary; post as PR comment if `--pr N` is set, otherwise print to stdout |

## Invariants

1. Runs full validation to generate the comment body
2. When `--pr` is omitted, prints markdown to stdout for piping
3. When `--pr N` is set, resolves repo and uses `gh pr comment` to post
4. Exits 1 if `gh` CLI fails or repo cannot be determined

## Behavioral Examples

### Scenario: Print to stdout

- **Given** `--pr` is not set
- **When** `cmd_comment` runs
- **Then** prints markdown summary to stdout

### Scenario: Post to PR

- **Given** `--pr 42` is set
- **When** `cmd_comment` runs
- **Then** posts comment on PR #42

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `gh` CLI not installed | Command fails with error |
| GitHub repo unresolvable | Exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns` |
| comment | `build_comment_body` |
| github | `resolve_repo` |
| validator | `validate_spec`, `compute_coverage` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync comment` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
