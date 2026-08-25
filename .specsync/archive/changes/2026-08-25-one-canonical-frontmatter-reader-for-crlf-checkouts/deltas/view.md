# View CRLF rendering delta

## MODIFIED

### SPEC SECTION Invariants

1. Four roles are supported: `dev`, `qa`, `product`, `agent`
2. Unknown roles return an error — never silently fall back
3. The `agent` role includes `status` and `agent_policy` from frontmatter in the output header
4. The `product` role appends companion `requirements.md` content if the file exists
5. `agent_policy` defaults to `"full-access"` if not set in frontmatter
6. Output includes a role-specific header line (e.g., `# ModuleName (dev view)`)
7. Section matching is based on `## ` heading prefixes
8. A CRLF-authored spec renders exactly as its LF twin does. `view_spec` reads the file with no normalization of its own, so a Windows clone with `core.autocrlf=true` used to fail with "Cannot parse frontmatter" on every spec in the project — on the one platform that ships a binary and is tested by no CI job. The tolerance belongs to `parser::parse_frontmatter`, and this module states the outcome it depends on rather than re-implementing it.
9. This module owns no frontmatter stripper. The companion `requirements.md` is stripped with `parser::strip_frontmatter`, the single canonical implementation. A local copy is how `view` and `change` came to disagree about CRLF and about a closing delimiter at end of file, with no error raised in either direction.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown role string | Returns `Err` listing valid roles |
| Spec file unreadable | Returns `Err` with read error description |
| Frontmatter absent or unterminated | Returns `Err` with parse error |
| CRLF line endings | Not an error condition — the spec and its companion render as they would on LF |
| Companion `requirements.md` missing or empty after stripping | Omitted; the rest of the view still renders |

### SPEC SECTION Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` for module name, status, and agent_policy; `strip_frontmatter` for the companion `requirements.md` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `view_spec`, `valid_roles` via `cmd_view` subcommand |
