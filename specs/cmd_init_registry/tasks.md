---
spec: cmd_init_registry.spec.md
---

## Tasks

- [ ] Add an integration test for `init-registry` (creation, `--name` override, and no-overwrite of an existing registry). No fixtures currently exist.

## Done

- [x] `cmd_init_registry` writes `specsync-registry.toml` via `registry::generate_registry`.
- [x] Project name resolution: `--name` → root dir name → `"project"`.
- [x] No-overwrite guard when the registry already exists.
- [x] Write-failure path prints an error and exits 1.

## Gaps

- No integration or inline unit tests target `src/commands/init_registry.rs`. Registry generation logic itself is covered in the `registry` module's tests.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
