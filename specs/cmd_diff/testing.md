---
spec: cmd_diff.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/diff.rs` | cargo test commands::diff | No inline tests found; add focused coverage for `cmd_diff`, `load_and_discover`, `get_exported_symbols`, `print_diff_markdown` before risky changes |
| `tests/integration.rs` | cargo test --test integration diff_shows_changes_since_base_ref | End-to-end fixture: `diff_shows_changes_since_base_ref` |
| `tests/integration.rs` | cargo test --test integration diff_no_changes_returns_empty | End-to-end fixture: `diff_no_changes_returns_empty` |
| `tests/integration.rs` | cargo test --test integration diff_detects_removed_exports | End-to-end fixture: `diff_detects_removed_exports` |
| `tests/integration.rs` | cargo test --test integration diff_human_readable_output | End-to-end fixture: `diff_human_readable_output` |
| `tests/integration.rs` | cargo test --test integration diff_detects_spec_file_only_changes | End-to-end fixture: `diff_detects_spec_file_only_changes` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| New export added | `src/auth.rs` added `pub fn verify_token()` since HEAD | `cmd_diff --base HEAD~1` runs | shows `auth` spec with "Added: `verify_token`" |
| No spec-tracked changes | only non-source files changed (e.g., README.md) | `cmd_diff` runs | prints "No spec-tracked source files changed since `{base}`." |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `git diff` fails (bad ref) | Exits with code 1 | Keep or add a focused assertion before changing this behavior |
| Changed file not in any spec | Listed under "Changed files not covered by any spec" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- diff --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/diff.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
