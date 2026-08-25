---
spec: view.spec.md
---

## Key Decisions

- **Hardcoded role → section map**: `sections_for_role` returns a fixed list of visible section names per role. Sections are matched with `heading.contains(allowed)` against `## ` headings split out by `split_sections`, so a visible parent section carries its `###` subsections along with it.
- **Fail fast on bad role**: role validation happens before the file is read; an unknown role returns an `Err` listing `valid_roles()` rather than defaulting.
- **Role-specific header**: emitted as `# {module} (view: {role})` only when the frontmatter has a module name.
- **Agent extras**: the `agent` role additionally emits `**Status:**` and `**Agent Policy:**` lines; a missing `agent_policy` renders the literal `not set (default: full-access)`.
- **Product companion inclusion**: the `product` role appends the sibling `requirements.md` (with its own frontmatter stripped via `parser::strip_frontmatter`) under a `## Requirements` heading, if the file exists and is non-empty.
- **No local frontmatter reader**: this module owns neither a parser nor a stripper. It had a private `strip_frontmatter` that was LF-only and rejected a closing delimiter at EOF, which is how it and `change`'s came to disagree with no error raised in either direction; both are gone in favour of the one in `parser` (#696).

## Files to Read First

- `src/view.rs` — entire module: `sections_for_role`, `view_spec`, `split_sections`.
- `src/parser.rs` — `parse_frontmatter` and `strip_frontmatter`, which decide everything this module knows about line endings and delimiters.

## Current Status

Stable and complete. Public API is `view_spec` and `valid_roles`. Invoked by the `cmd_view` subcommand.

## Notes

- Depends on `parser::parse_frontmatter` for module name, status, and `agent_policy`.
- `view_spec` reads the file with `fs::read_to_string` and does not normalize. That was fine only
  because the parser normalizes; before it did, a Windows clone with `core.autocrlf=true` made
  this module fail with "Cannot parse frontmatter" on every spec in the project — the whole
  `cmd_view` surface, on the one platform we ship a binary for and no CI job tests.
- Output is always a markdown `String`; no alternative formats.
- `split_sections` only recognizes `## ` (level-2) headings as section boundaries.
