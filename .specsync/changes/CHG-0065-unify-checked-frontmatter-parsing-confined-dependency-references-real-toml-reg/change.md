---
id: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
state: approved
type: bug_fix
base_commit: 01b79e910ff1d0d9e00981a2a61c4fa0b6a22030
---

# Unify checked frontmatter parsing, confined dependency references, real TOML registries, authenticated resolution, and fail-closed dependency gates for issues 413, 419, 422, 436, and 444

## Intent

Unify checked frontmatter parsing, confined dependency references, real TOML registries, authenticated resolution, and fail-closed dependency gates for issues 413, 419, 422, 436, and 444

## Affected Canonical Specs

- `types`
- `parser`
- `registry`
- `github`
- `deps`
- `cmd_deps`
- `validator`
- `scoring`
- `cmd_resolve`
- `cli`
- `commands`
- `cmd_check`
- `mcp`

## Acceptance Criteria

- Checked frontmatter rejects duplicate keys and malformed known fields while preserving valid extensions
- One confined DependencyRef parser produces identical diagnostics across check deps resolve scoring and MCP
- Registries parse real TOML in canonical specs and documented modules forms while continuing to emit canonical specs
- Remote registry and spec fetches authenticate safely and never report false success
- Dependency resolution and coverage gates exit consistently and emit parseable structured output

## No-spec Rationale

Not applicable
