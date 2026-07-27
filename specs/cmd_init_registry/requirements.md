---
spec: cmd_init_registry.spec.md
---

## User Stories

- As a maintainer of a multi-project workspace, I want `specsync init-registry` to generate a `specsync-registry.toml` listing my specs so that other projects can reference them.
- As a developer, I want the registry's project name to default to the directory name but be overridable with `--name` so that the registry reads naturally.
- As a developer, I want init-registry to be safe to re-run so that it never overwrites an existing registry.

## Acceptance Criteria

- `cmd_init_registry` writes `specsync-registry.toml` at the project root using `registry::generate_registry(root, project_name, &config.specs_dir)`.
- The project name is `name` when provided; otherwise the root directory's file name, falling back to `"project"` when that cannot be determined.
- If `specsync-registry.toml` already exists, the command prints a message and returns without writing.
- On success, prints "Created specsync-registry.toml".
- Empty/whitespace-only names fail without output files.
- Hostile names and module keys serialize as literal TOML values/keys.
- Existing valid registries are unchanged visible no-ops; invalid existing files fail visibly.
- Structured output exposes `success`, `created`, and `unchanged`.

## Constraints

- Reads configuration via `config::load_config` to obtain `specs_dir`; performs no spec validation itself.
- Must not panic on expected error conditions; a write failure prints an error and exits 1.

## Out of Scope

- Discovering and rendering registry entries (owned by `registry::generate_registry`).
- Updating or merging into an existing registry (existing file is left untouched).
- Interactive prompts, GUI, or web output.

### REQ-cmd-init-registry-001

The registry initialization command SHALL create the canonical registry once from discovered specs and SHALL not overwrite existing state.

Acceptance Criteria
- `cmd_init_registry` writes `specsync-registry.toml` at the project root using `registry::generate_registry(root, project_name, &config.specs_dir)`.
- The project name is `name` when provided; otherwise the root directory's file name, falling back to `"project"` when that cannot be determined.
- If `specsync-registry.toml` already exists, the command prints a message and returns without writing.
- On success, prints "Created specsync-registry.toml".

### REQ-cmd-init-registry-002

Registry initialization SHALL validate inputs and selected configuration, serialize TOML safely, and report every no-op or failure truthfully.

Acceptance Criteria

- Blank names fail before reading/writing registry state.
- Selected config syntax and known path-field shapes are checked before creation.
- Generated names, module keys, and paths round-trip literally without TOML injection.
- Existing valid registries remain byte-identical and report `created = false`, `unchanged = true`.
- Existing malformed/inert registries fail without overwrite.
- JSON/Markdown/GitHub/table/CSV outputs distinguish created, unchanged, and failed outcomes.
