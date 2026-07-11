## ADDED

### REQUIREMENT REQ-cmd-init-registry-001

The registry initialization command SHALL create the canonical registry once from discovered specs and SHALL not overwrite existing state.

Acceptance Criteria
- `cmd_init_registry` writes `specsync-registry.toml` at the project root using `registry::generate_registry(root, project_name, &config.specs_dir)`.
- The project name is `name` when provided; otherwise the root directory's file name, falling back to `"project"` when that cannot be determined.
- If `specsync-registry.toml` already exists, the command prints a message and returns without writing.
- On success, prints "Created specsync-registry.toml".
