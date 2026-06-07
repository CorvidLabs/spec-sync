---
spec: cmd_init.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/init.rs` | cargo test commands::init | No inline tests found; add focused coverage for `cmd_init`, `ensure_hashes_gitignored`, `detect_source_dirs` before risky changes |
| `tests/integration.rs` | cargo test --test integration init_creates_config_file | End-to-end fixture: `init_creates_config_file` |
| `tests/integration.rs` | cargo test --test integration init_does_not_overwrite_existing_config | End-to-end fixture: `init_does_not_overwrite_existing_config` |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_src_dir | End-to-end fixture: `init_auto_detects_src_dir` |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_lib_dir | End-to-end fixture: `init_auto_detects_lib_dir` |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_multiple_dirs | End-to-end fixture: `init_auto_detects_multiple_dirs` |
| `tests/integration.rs` | cargo test --test integration init_ignores_node_modules_and_hidden_dirs | End-to-end fixture: `init_ignores_node_modules_and_hidden_dirs` |
| `tests/integration.rs` | cargo test --test integration init_falls_back_to_src_when_no_source_files | End-to-end fixture: `init_falls_back_to_src_when_no_source_files` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_init_creates_config | End-to-end fixture: `mcp_tool_init_creates_config` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| First init | no `specsync.json` exists | `cmd_init(root)` runs | creates config with detected source dirs |
| Config exists | `specsync.json` already exists | `cmd_init(root)` runs | prints message and returns without changes |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| File write fails | Exits 1 | Keep or add a focused assertion before changing this behavior |
| No source dirs detected | Creates config with empty `sourceDirs` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- init --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/init.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
