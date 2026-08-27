## MODIFIED

### SPEC SECTION Invariants

1. Four roles are supported: `dev`, `qa`, `product`, `agent`
2. Unknown roles return an error — never silently fall back
3. The `agent` role includes `status` and `agent_policy` from frontmatter in the output header
4. The `product` role appends companion `requirements.md` content if the file exists
5. `agent_policy` defaults to `"full-access"` if not set in frontmatter
6. Output includes a role-specific header line (e.g., `# ModuleName (dev view)`)
7. Section matching is based on `## ` heading prefixes
8. A CRLF-authored spec renders exactly as its LF twin does. `view_spec` reads the file with no normalization of its own, so a Windows clone with `core.autocrlf=true` used to fail with "Cannot parse frontmatter" on every spec in the project — on Windows, for which SpecSync then published a binary that no ordinary CI job ever exercised. SpecSync 6.0 publishes no Windows binary, and the tolerance is unaffected by that: a CRLF checkout reaches a Linux or macOS reader through any teammate who authored on Windows. The tolerance belongs to `parser::parse_frontmatter`, and this module states the outcome it depends on rather than re-implementing it.
9. This module owns no frontmatter stripper. The companion `requirements.md` is stripped with `parser::strip_frontmatter`, the single canonical implementation. A local copy is how `view` and `change` came to disagree about CRLF and about a closing delimiter at end of file, with no error raised in either direction.
