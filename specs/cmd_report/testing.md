---
spec: cmd_report.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/report.rs` | cargo test commands::report | No inline tests found; add focused coverage for `cmd_report`, `load_and_discover`, `parse_frontmatter`, `OutputFormat` before risky changes |
| `tests/integration.rs` | cargo test --test integration coverage_full_reports_100 | End-to-end fixture: `coverage_full_reports_100` |
| `tests/integration.rs` | cargo test --test integration invalid_frontmatter_reports_error | End-to-end fixture: `invalid_frontmatter_reports_error` |
| `tests/integration.rs` | cargo test --test integration missing_required_sections_reports_error | End-to-end fixture: `missing_required_sections_reports_error` |
| `tests/integration.rs` | cargo test --test integration missing_frontmatter_fields_reports_error | End-to-end fixture: `missing_frontmatter_fields_reports_error` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Stale spec detection | `src/auth.rs` has 12 commits since `specs/auth/auth.spec.md` was last modified | `cmd_report` runs with default `stale_threshold: 5` | auth module is flagged as stale with "12 commits behind" |
| All modules healthy | all specs are up to date and complete | `cmd_report` runs | every module shows "no" for Stale and Incomplete columns |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Git not available or not a git repo | Staleness detection gracefully returns 0 (not stale) | Keep or add a focused assertion before changing this behavior |
| Spec references a file that doesn't exist | File is skipped in staleness calculation | Keep or add a focused assertion before changing this behavior |
| No spec files found | Prints "no specs found" and exits 0 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- report --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/report.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
