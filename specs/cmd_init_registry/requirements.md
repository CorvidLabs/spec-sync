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

## Constraints

- Reads configuration via `config::load_config` to obtain `specs_dir`; performs no spec validation itself.
- Must not panic on expected error conditions; a write failure prints an error and exits 1.

## Out of Scope

- Discovering and rendering registry entries (owned by `registry::generate_registry`).
- Updating or merging into an existing registry (existing file is left untouched).
- Interactive prompts, GUI, or web output.

### REQ-cmd-init-registry-001

The `cmd_init_registry` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

