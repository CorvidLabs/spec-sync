---
spec: cmd_coverage.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/coverage.rs` | cargo test commands::coverage | No inline tests found; add focused coverage for `cmd_coverage`, `IgnoreRules::load`, `compute_coverage` before risky changes |
| `tests/integration.rs` | cargo test --test integration coverage_full_reports_100 | End-to-end fixture: `coverage_full_reports_100` |
| `tests/integration.rs` | cargo test --test integration coverage_partial_lists_unspecced_files | End-to-end fixture: `coverage_partial_lists_unspecced_files` |
| `tests/integration.rs` | cargo test --test integration coverage_shows_unspecced_modules | End-to-end fixture: `coverage_shows_unspecced_modules` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration strict_on_coverage_subcommand | End-to-end fixture: `strict_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_coverage_returns_metrics | End-to-end fixture: `mcp_tool_coverage_returns_metrics` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Full coverage | all source files claimed by specs | `cmd_coverage` runs | prints 100% with green check marks |
| Below threshold | 58% coverage, `--require-coverage 80` | `cmd_coverage` runs | lists uncovered files and exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Coverage below threshold | Exits 1 with details | Keep or add a focused assertion before changing this behavior |
| No specs found | Prints suggestion, exits 0 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- coverage --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/coverage.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
