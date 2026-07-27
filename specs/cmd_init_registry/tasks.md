---
spec: cmd_init_registry.spec.md
---

## Tasks

- [x] Add integration coverage for `init-registry` creation and no-overwrite behavior — Evidence: `init_registry_uses_v4_path_in_migrated_project`, `init_registry_keeps_legacy_path_for_legacy_project`, and `init_registry_is_idempotent_for_v4_registry`.

## Done

- [x] `cmd_init_registry` writes `specsync-registry.toml` via `registry::generate_registry`.
- [x] Project name resolution: `--name` → root dir name → `"project"`.
- [x] No-overwrite guard when the registry already exists.
- [x] Write-failure path prints an error and exits 1.
- [x] Integration coverage for `--name`, structured create/no-op output, blank-name rejection, and hostile TOML serialization.
- [x] Validate existing registries/config before claiming success or writing.

## Gaps

- No platform-specific permission-denied fixture; portable blocking-path and create-new behavior cover deterministic write failures.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
