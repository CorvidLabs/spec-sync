---
spec: cmd_init_registry.spec.md
---

## Key Decisions

- The command is a thin wrapper around `registry::generate_registry`; all spec discovery and TOML rendering live in the `registry` module. This command only resolves the project name, guards against overwrite, and writes the file.
- Project name defaults to the root directory's file name (`root.file_name()`), with `"project"` as a last-resort fallback, so the registry is usable even when run from an oddly-named or root path.
- Like `init`, it refuses to overwrite an existing `specsync-registry.toml` and simply reports that it already exists.
- Name validation happens before the existing-file guard so invalid CLI input is never silently ignored.
- Existing registries are parsed before reporting an idempotent no-op; malformed/inert files fail visibly and remain untouched.
- Selected config syntax/path-field shapes are checked before generation.
- All output formats render one outcome record with explicit created/unchanged/failure state.

## Files to Read First

- `src/commands/init_registry.rs` — the whole command.
- `src/registry.rs` — `generate_registry(root, project_name, specs_dir)`, which discovers specs and renders the TOML.
- `src/config.rs` — `load_config` (provides `specs_dir`).

## Current Status

Implemented and stable. Integration coverage exercises creation, valid existing no-op, invalid names, hostile TOML values/keys, and JSON output.

## Notes

- Part of the command layer — orchestrates the `registry` module rather than containing domain logic.
