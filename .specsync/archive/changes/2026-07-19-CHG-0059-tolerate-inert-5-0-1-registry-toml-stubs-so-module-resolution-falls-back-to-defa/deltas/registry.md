## ADDED

### REQUIREMENT REQ-registry-002

Local registry loading SHALL treat inert 5.0.1-era empty registry stubs as absent while still failing closed on unparsable real registries.

Acceptance Criteria

- A local registry file with no registry `name` and no `[specs]` module mappings is classified as an inert stub and loaded as absent.
- The characteristic 5.0.1 placeholder (`version = 1` plus an empty `[modules]` table) is inert.
- A named registry loads successfully even when `[specs]` is empty.
- A file that is not inert but cannot parse as a named registry fails closed through the Result-based local loader.
- Best-effort `load_registry` continues to return `None` for missing, inert, and unparsable content.

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `RemoteRegistry` | A parsed remote registry with project name and list of (module, spec_path) entries |
| `RemoteSpec` | Fetched remote spec content with parsed module, status, depends_on, exports, and body |

**Exported RemoteRegistry Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `has_spec` | `module: &str` | `bool` | Check whether a module name exists in this registry |
| `spec_path` | `module: &str` | `Option<&str>` | Get the spec file path for a module from the registry |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `fetch_remote_registry` | `repo: &str` | `Result<RemoteRegistry, String>` | Fetch `specsync-registry.toml` from a GitHub repo's default branch via raw content URL |
| `fetch_remote_spec` | `repo: &str, spec_path: &str` | `Result<String, String>` | Fetch a spec file's raw content from a GitHub repo |
| `parse_remote_spec` | `module: &str, content: &str` | `Option<RemoteSpec>` | Parse fetched spec content into metadata for verification |
| `local_registry_path` | `root: &Path` | `PathBuf` | Resolve the local registry path — prefers v4 `.specsync/registry.toml`, falls back to legacy root-level `specsync-registry.toml` for un-migrated 3.x projects |
| `is_inert_legacy_registry_stub` | `content: &str` | `bool` | Return true when content has no registry `name` and no `[specs]` mappings (including empty 5.0.1 `[modules]` placeholders) |
| `load_local_registry` | `root: &Path` | `Result<Option<RegistryEntry>, String>` | Load the local registry as `Ok(None)` when missing or inert, `Ok(Some(entry))` when parsable, and `Err` when a non-inert file fails to parse |
| `load_registry` | `root: &Path` | `Option<RegistryEntry>` | Best-effort load from the local registry file resolved by `local_registry_path`; returns `None` for missing, inert, or unparsable content |
| `generate_registry` | `root, project_name, specs_dir` | `String` | Generate registry TOML content by scanning for spec files |
| `register_module` | `root, module_name, spec_rel_path` | `bool` | Append a module entry to the registry file resolved by `local_registry_path`; returns false if already exists or file missing |

### SPEC SECTION Invariants

1. Remote registry fetch uses a 10-second HTTP timeout
2. Registry TOML format: `[registry]` section with `name`, `[specs]` section with `module = "path"` entries
3. `generate_registry` skips template files (names starting with `_`)
4. Module names are extracted from spec frontmatter, not file paths
5. Generated registry entries are sorted alphabetically by module name
6. `RemoteRegistry::has_spec` performs exact module name matching
7. TOML parsing is zero-dependency — uses line-by-line string parsing
8. Local registry resolution prefers the v4 `.specsync/registry.toml` location; the legacy root-level `specsync-registry.toml` is only used for un-migrated 3.x layouts
9. Inert 5.0.1-era stubs (no registry name and no `[specs]` mappings) are treated as absent; non-inert unparsable registries fail closed through `load_local_registry`

### SPEC SECTION Behavioral Examples

**Scenario: Fetch remote registry**

- **Given** a GitHub repo "corvid-labs/algochat" with a `specsync-registry.toml` at root
- **When** `fetch_remote_registry("corvid-labs/algochat")` is called
- **Then** returns `Ok(RemoteRegistry)` with parsed module-to-path mappings

**Scenario: Generate registry from local specs**

- **Given** specs at `specs/auth/auth.spec.md` and `specs/messaging/messaging.spec.md`
- **When** `generate_registry(root, "myproject", "specs")` is called
- **Then** returns TOML string with `[registry]\nname = "myproject"\n\n[specs]\nauth = "specs/auth/auth.spec.md"\nmessaging = "specs/messaging/messaging.spec.md"\n`

**Scenario: Check module existence**

- **Given** a `RemoteRegistry` with specs for "auth" and "messaging"
- **When** `has_spec("auth")` is called
- **Then** returns `true`

**Scenario: Tolerate inert 5.0.1 registry stub**

- **Given** a local `.specsync/registry.toml` containing only `version = 1` and an empty `[modules]` table
- **When** `load_local_registry(root)` is called
- **Then** returns `Ok(None)` so callers fall back to conventional module paths

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| HTTP request fails | Error: "HTTP request failed: {details}" |
| Repo has no registry file | Error: "HTTP 404 — {repo} may not have a specsync-registry.toml" |
| Malformed TOML (no name) | `parse_registry` returns `None` |
| Local registry file unreadable | `load_registry` returns `None`; `load_local_registry` returns `Err` |
| Inert legacy stub (no name, no `[specs]` mappings) | `load_local_registry` returns `Ok(None)`; `load_registry` returns `None` |
| Non-inert unparsable local registry | `load_local_registry` returns `Err`; `load_registry` returns `None` |
