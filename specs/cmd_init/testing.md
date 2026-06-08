---
spec: cmd_init.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/init.rs` (inline) | cargo test --lib commands::init | `adds_entry_to_missing_gitignore` — creates `.gitignore` with the entry. |
| `src/commands/init.rs` (inline) | cargo test --lib commands::init | `is_idempotent_when_entry_already_present` — entry count stays 1, content untouched. |
| `src/commands/init.rs` (inline) | cargo test --lib commands::init | `errors_when_gitignore_path_is_unwritable` — `.gitignore` made a dir; returns `Err` containing "Failed to update .gitignore". |
| `tests/integration.rs` | cargo test --test integration init_creates_config_file | Writes `specsync.json` containing `specsDir`/`sourceDirs`/`requiredSections`. |
| `tests/integration.rs` | cargo test --test integration init_does_not_overwrite_existing_config | Existing `{"specsDir":"custom"}` is preserved; prints "already exists". |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_src_dir | `sourceDirs == ["src"]`. |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_lib_dir | `sourceDirs == ["lib"]`. |
| `tests/integration.rs` | cargo test --test integration init_auto_detects_multiple_dirs | `sourceDirs == ["lib", "src"]` (sorted). |
| `tests/integration.rs` | cargo test --test integration init_ignores_node_modules_and_hidden_dirs | Only `app` detected; `node_modules`/`.cache` ignored. |
| `tests/integration.rs` | cargo test --test integration init_falls_back_to_src_when_no_source_files | Empty project falls back to `sourceDirs == ["src"]`. |
| `tests/integration.rs` | cargo test --test integration mcp_tool_init_creates_config | MCP `init` tool creates the config file. |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| First init | empty temp project | `specsync init` | Writes `specsync.json` with detected dirs; prints "Created specsync.json" and the detected dirs. |
| Config already exists | `specsync.json` present | `specsync init` | Prints "already exists" and leaves the file untouched. |
| TOML config exists | `.specsync.toml` present | `specsync init` | Prints ".specsync.toml already exists" and no-ops. |
| Missing `.gitignore` | no `.gitignore` | `specsync init` | Creates `.gitignore` and prints "Added .specsync/hashes.json to .gitignore". |
| Entry already gitignored | `.gitignore` already lists the entry | `specsync init` | No duplicate appended (no extra line printed). |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `specsync.json` write fails | Exits 1 | Add a focused assertion before changing the write path. |
| `.gitignore` write fails | Non-fatal: prints a `warning:` and init still succeeds | Covered indirectly by `errors_when_gitignore_path_is_unwritable` on the helper. |
| No source files present | Falls back to `sourceDirs == ["src"]` (not empty) | Covered by `init_falls_back_to_src_when_no_source_files`. |
| Gitignore entry already present | Idempotent — no duplicate | Covered by `is_idempotent_when_entry_already_present`. |

## Reviewer Checklist

- Run `cargo run -- init --help` and confirm the documented behavior is intact.
- Run `cargo test --lib commands::init` and `cargo test --test integration init_` before the full suite when changing `src/commands/init.rs`.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output.
- If an error/warning message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
