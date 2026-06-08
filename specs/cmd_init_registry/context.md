---
spec: cmd_init_registry.spec.md
---

## Key Decisions

- The command is a thin wrapper around `registry::generate_registry`; all spec discovery and TOML rendering live in the `registry` module. This command only resolves the project name, guards against overwrite, and writes the file.
- Project name defaults to the root directory's file name (`root.file_name()`), with `"project"` as a last-resort fallback, so the registry is usable even when run from an oddly-named or root path.
- Like `init`, it refuses to overwrite an existing `specsync-registry.toml` and simply reports that it already exists.

## Files to Read First

- `src/commands/init_registry.rs` — the whole command.
- `src/registry.rs` — `generate_registry(root, project_name, specs_dir)`, which discovers specs and renders the TOML.
- `src/config.rs` — `load_config` (provides `specs_dir`).

## Current Status

Implemented and stable. No tests target this file directly; the registry-rendering logic is covered by the `registry` module's own tests.

## Notes

- Part of the command layer — orchestrates the `registry` module rather than containing domain logic.
