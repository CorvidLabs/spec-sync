---
spec: registry.spec.md
---

## User Stories

- As a developer with cross-project dependencies, I want to generate a `specsync-registry.toml` that advertises my project's specs so that other projects can reference them
- As a developer, I want to resolve cross-project spec references by fetching remote registries from GitHub so that I can validate that dependencies are documented
- As a team lead, I want `specsync init-registry` to auto-generate the registry from existing specs so that publishing is a single command
- As a developer, I want remote registry fetches to time out quickly so that a missing or slow GitHub repo doesn't block my workflow

## Acceptance Criteria

- `generate_registry` produces valid TOML with `[registry]` name and `[specs]` module-path pairs
- Generated registry skips template files (names starting with `_`)
- Module names are read from spec frontmatter, not inferred from file paths
- Registry entries are sorted alphabetically by module name
- `fetch_remote_registry` uses HTTPS with a 10-second timeout
- `RemoteRegistry::has_spec` performs exact module name matching
- TOML parsing is zero-dependency (line-by-line string parsing)
- HTTP errors and timeouts produce clear error messages

## Constraints

- Remote fetches use raw.githubusercontent.com — no GitHub API token required
- Registry format must be simple TOML that humans can read and edit
- No caching of remote registries (always fetches fresh)

## Out of Scope

- Registry hosting or publishing service
- Version negotiation between registries
- Authentication for private GitHub repositories
- Recursive resolution of transitive cross-project dependencies
