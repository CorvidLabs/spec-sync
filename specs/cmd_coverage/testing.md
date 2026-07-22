---
spec: cmd_coverage.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/coverage.rs` | cargo test commands::coverage | Command wrapper has no inline tests; exercised end-to-end via the integration fixtures below. Note: this wrapper uses `IgnoreRules::default()`, not `IgnoreRules::load` |
| `tests/integration.rs` | cargo test --test integration coverage_full_reports_100 | End-to-end fixture: `coverage_full_reports_100` |
| `tests/integration.rs` | cargo test --test integration coverage_partial_lists_unspecced_files | End-to-end fixture: `coverage_partial_lists_unspecced_files` |
| `tests/integration.rs` | cargo test --test integration coverage_shows_unspecced_modules | End-to-end fixture: `coverage_shows_unspecced_modules` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration strict_on_coverage_subcommand | End-to-end fixture: `strict_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_coverage_returns_metrics | End-to-end fixture: `mcp_tool_coverage_returns_metrics` |
| `tests/integration/commands.rs` | cargo test --test integration malformed_gradle_is_inconclusive_for_coverage_gating_commands | Malformed Gradle discovery exits 1 with parseable `valid: false` / `inconclusive: true` JSON |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Full coverage | all source files claimed by specs | `cmd_coverage` runs | prints 100% with green check marks |
| Below threshold | partial coverage, `--require-coverage 80` | `cmd_coverage` runs | lists uncovered files and exits 1 |
| JSON metrics dump | `--format json` with trustworthy discovery | `cmd_coverage` runs | emits coverage keys (`file_coverage`, `loc_coverage`, `uncovered_files`, …); configured gates determine exit status |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Coverage below threshold | Exits 1 with details | Keep or add a focused assertion before changing this behavior |
| No specs found | Prints suggestion, exits 0 | Keep or add a focused assertion before changing this behavior |
| Malformed Gradle settings | Emits valid structured inconclusive JSON and exits 1 | Covered by `malformed_gradle_is_inconclusive_for_coverage_gating_commands` |

## Reviewer Checklist

- Run `cargo run -- coverage --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/coverage.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
