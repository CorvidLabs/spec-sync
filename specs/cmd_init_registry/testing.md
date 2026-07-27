---
spec: cmd_init_registry.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/init_registry.rs` | `cargo test --test integration init_registry_` | Creation, no-overwrite, structured outcomes, blank-name rejection, hostile serialization |
| `src/registry.rs` | `cargo test registry::` | Checked parsing, safe generation, entry discovery, exact module identity |

## Coverage Gaps

- Permission-denied behavior remains platform-dependent; deterministic blocking/create-new failures provide portable coverage.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Generate registry | a project with N specs, no existing registry | `specsync init-registry` | Writes `specsync-registry.toml` (one entry per discovered spec); prints "Created specsync-registry.toml". |
| Name override | any project | `specsync init-registry --name my-lib` | Registry's project name is `my-lib` instead of the directory name. |
| Registry exists | `specsync-registry.toml` already present | `specsync init-registry` | Prints "specsync-registry.toml already exists" and writes nothing. |
| Hostile name/key | quotes/newlines in name and `api.v2` module | `specsync init-registry --name ...` | Valid TOML, literal values, no injected mapping |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Registry already exists | Visible unchanged success only when valid | `init_registry_json_reports_create_and_existing_noop_truthfully` |
| `--name` provided | Used literally as the project name | `init_registry_serializes_hostile_name_and_module_key_as_valid_toml` |
| Blank `--name` | Exit 1, structured error, no output file | `init_registry_rejects_blank_name_as_structured_failure_without_output_file` |
| Name not provided | Defaults to root dir name, then `"project"` | Add a fixture before changing the fallback chain. |
| Write fails | Prints error, exits 1 | Add a focused assertion before changing the write path. |

## Reviewer Checklist

- Run `cargo run -- init-registry --help` and confirm `--name` is present.
- For changes to discovered entries or TOML shape, run the `registry` module's tests — that is where rendering lives.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
