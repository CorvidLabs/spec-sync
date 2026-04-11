---
module: registry
version: 2
status: stable
files:
  - src/registry.rs
db_tables: []
tracks: [52]
depends_on:
  - specs/types/types.spec.md
---

# Registry

## Purpose

Manages cross-project spec registries for dependency resolution. Generates `specsync-registry.toml` from local spec files, fetches remote registries from GitHub repos via HTTPS, and parses the TOML registry format using zero-dependency parsing.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `RemoteRegistry` | A parsed remote registry with project name and list of (module, spec_path) entries |
| `RemoteSpec` | Fetched remote spec content with parsed module, status, depends_on, exports, and body |

### Exported RemoteRegistry Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `has_spec` | `module: &str` | `bool` | Check whether a module name exists in this registry |
| `spec_path` | `module: &str` | `Option<&str>` | Get the spec file path for a module from the registry |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `fetch_remote_registry` | `repo: &str` | `Result<RemoteRegistry, String>` | Fetch `specsync-registry.toml` from a GitHub repo's default branch via raw content URL |
| `fetch_remote_spec` | `repo: &str, spec_path: &str` | `Result<String, String>` | Fetch a spec file's raw content from a GitHub repo |
| `parse_remote_spec` | `module: &str, content: &str` | `Option<RemoteSpec>` | Parse fetched spec content into metadata for verification |
| `load_registry` | `root: &Path` | `Option<RegistryEntry>` | Load a registry from a local `specsync-registry.toml` file |
| `generate_registry` | `root, project_name, specs_dir` | `String` | Generate registry TOML content by scanning for spec files |
| `register_module` | `root, module_name, spec_rel_path` | `bool` | Append a module entry to the registry file; returns false if already exists or file missing |

## Invariants

1. Remote registry fetch uses a 10-second HTTP timeout
2. Registry TOML format: `[registry]` section with `name`, `[specs]` section with `module = "path"` entries
3. `generate_registry` skips template files (names starting with `_`)
4. Module names are extracted from spec frontmatter, not file paths
5. Generated registry entries are sorted alphabetically by module name
6. `RemoteRegistry::has_spec` performs exact module name matching
7. TOML parsing is zero-dependency — uses line-by-line string parsing

## Behavioral Examples

### Scenario: Fetch remote registry

- **Given** a GitHub repo "corvid-labs/algochat" with a `specsync-registry.toml` at root
- **When** `fetch_remote_registry("corvid-labs/algochat")` is called
- **Then** returns `Ok(RemoteRegistry)` with parsed module-to-path mappings

### Scenario: Generate registry from local specs

- **Given** specs at `specs/auth/auth.spec.md` and `specs/messaging/messaging.spec.md`
- **When** `generate_registry(root, "myproject", "specs")` is called
- **Then** returns TOML string with `[registry]\nname = "myproject"\n\n[specs]\nauth = "specs/auth/auth.spec.md"\nmessaging = "specs/messaging/messaging.spec.md"\n`

### Scenario: Check module existence

- **Given** a `RemoteRegistry` with specs for "auth" and "messaging"
- **When** `has_spec("auth")` is called
- **Then** returns `true`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| HTTP request fails | Error: "HTTP request failed: {details}" |
| Repo has no registry file | Error: "HTTP 404 — {repo} may not have a specsync-registry.toml" |
| Malformed TOML (no name) | `parse_registry` returns `None` |
| Local registry file unreadable | `load_registry` returns `None` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `RegistryEntry` |
| ureq | HTTP client for fetching remote registries |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `fetch_remote_registry`, `generate_registry`, `load_registry` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-04-07 | Document register_module function |
| 2026-04-10 | v2: Added `fetch_remote_spec`, `parse_remote_spec`, `RemoteSpec`, `spec_path` for cross-repo content verification |
