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
- Registry parsing uses a real TOML parser and validates supported field types and shapes
- Both emitted `[specs]` mappings and documented `[[modules]]` entries are accepted
- Generated registry names, keys, and paths are encoded without TOML injection
- Invalid or ambiguous non-inert registries fail closed with a diagnostic
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

### REQ-registry-001

The registry module SHALL generate deterministic local registries and safely resolve supported remote registry references.

Acceptance Criteria
- `generate_registry` produces valid TOML with `[registry]` name and `[specs]` module-path pairs
- Generated registry skips template files (names starting with `_`)
- Module names are read from spec frontmatter, not inferred from file paths
- Registry entries are sorted alphabetically by module name
- `fetch_remote_registry` uses HTTPS with a 10-second timeout
- `RemoteRegistry::has_spec` performs exact module name matching
- Registry parsing uses a real TOML parser and validates supported field types and shapes
- Both emitted `[specs]` mappings and documented `[[modules]]` entries are accepted
- Generated registry names, keys, and paths are encoded without TOML injection
- Invalid or ambiguous non-inert registries fail closed with a diagnostic
- HTTP errors and timeouts produce clear error messages

### REQ-registry-002

Local registry loading SHALL treat inert 5.0.1-era empty registry stubs as absent while still failing closed on unparsable real registries.

Acceptance Criteria

- A local registry file with no registry `name` and no `[specs]` module mappings is classified as an inert stub and loaded as absent.
- The characteristic 5.0.1 placeholder (`version = 1` plus an empty `[modules]` table) is inert.
- A named registry loads successfully even when `[specs]` is empty.
- A file that is not inert but cannot parse as a named registry fails closed through the Result-based local loader.
- Best-effort `load_registry` continues to return `None` for missing, inert, and unparsable content.

### REQ-registry-003

The registry module SHALL preserve every valid mapping in either supported TOML shape and reject invalid, ambiguous, or injection-prone registry content.

Acceptance Criteria

- A named `[specs]` table maps each string-valued module key to its declared spec path.
- A named `[[modules]]` array maps each non-empty `name` to its non-empty `spec` path.
- Malformed TOML and wrong known-field types or shapes fail closed.
- Duplicate module names across supported shapes fail closed instead of selecting one mapping.
- Generated project names, module keys, and paths round-trip as literal values without creating extra TOML keys or tables.
- The nameless, mapping-free 5.0.1 `[modules]` table placeholder remains inert for compatibility.
