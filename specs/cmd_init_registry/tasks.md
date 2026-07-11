---
spec: cmd_init_registry.spec.md
---

## Tasks

- [x] Add integration coverage for `init-registry` creation and no-overwrite behavior — Evidence: `init_registry_uses_v4_path_in_migrated_project`, `init_registry_keeps_legacy_path_for_legacy_project`, and `init_registry_is_idempotent_for_v4_registry`.

## Post-5.0 Test Debt

- [ ] Add integration coverage for the `init-registry --name` override.

## Done

- [x] `cmd_init_registry` writes `specsync-registry.toml` via `registry::generate_registry`.
- [x] Project name resolution: `--name` → root dir name → `"project"`.
- [x] No-overwrite guard when the registry already exists.
- [x] Write-failure path prints an error and exits 1.

## Gaps

- No integration or inline unit tests target `src/commands/init_registry.rs`. Registry generation logic itself is covered in the `registry` module's tests.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
