## ADDED

### REQUIREMENT REQ-registry-001

The registry module SHALL generate deterministic local registries and safely resolve supported remote registry references.

Acceptance Criteria
- `generate_registry` produces valid TOML with `[registry]` name and `[specs]` module-path pairs
- Generated registry skips template files (names starting with `_`)
- Module names are read from spec frontmatter, not inferred from file paths
- Registry entries are sorted alphabetically by module name
- `fetch_remote_registry` uses HTTPS with a 10-second timeout
- `RemoteRegistry::has_spec` performs exact module name matching
- TOML parsing is zero-dependency (line-by-line string parsing)
- HTTP errors and timeouts produce clear error messages
