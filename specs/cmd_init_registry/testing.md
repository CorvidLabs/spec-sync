---
spec: cmd_init_registry.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/init_registry.rs` | (none) | No inline `#[cfg(test)]` module and no integration fixtures target this command. |
| `src/registry.rs` | cargo test registry | The `registry` module owns tests for `generate_registry`; verify entry discovery and TOML shape there. |

## Coverage Gaps

- No fixture creates a registry and asserts its contents. Add an integration test that: (1) runs `init-registry` in a project with a few specs and checks `specsync-registry.toml` is written with one entry per spec, (2) verifies `--name` sets the project name, and (3) verifies a second run does not overwrite the existing file.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Generate registry | a project with N specs, no existing registry | `specsync init-registry` | Writes `specsync-registry.toml` (one entry per discovered spec); prints "Created specsync-registry.toml". |
| Name override | any project | `specsync init-registry --name my-lib` | Registry's project name is `my-lib` instead of the directory name. |
| Registry exists | `specsync-registry.toml` already present | `specsync init-registry` | Prints "specsync-registry.toml already exists" and writes nothing. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Registry already exists | Early return, no overwrite | Add a fixture before changing the overwrite guard. |
| `--name` provided | Used as the project name | Add a fixture before changing name resolution. |
| Name not provided | Defaults to root dir name, then `"project"` | Add a fixture before changing the fallback chain. |
| Write fails | Prints error, exits 1 | Add a focused assertion before changing the write path. |

## Reviewer Checklist

- Run `cargo run -- init-registry --help` and confirm `--name` is present.
- For changes to discovered entries or TOML shape, run the `registry` module's tests — that is where rendering lives.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
