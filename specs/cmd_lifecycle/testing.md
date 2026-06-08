---
spec: cmd_lifecycle.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| Status mutation | cargo test commands::lifecycle:: | `update_status_in_content_replaces_status_line`, `update_status_preserves_rest_of_frontmatter`, `update_status_returns_none_when_no_status_line` |
| Lifecycle graph | cargo test commands::lifecycle:: | `spec_status_next`, `spec_status_prev`, `spec_status_valid_transitions`, `spec_status_can_transition_to` |
| Guard matching | cargo test commands::lifecycle:: | `find_guards_specific_and_wildcard`, `find_guards_ascii_arrow` |
| History log + age | cargo test commands::lifecycle:: | `append_lifecycle_log_new`, `append_lifecycle_log_existing`, `days_since_date_same_day_is_zero`, `days_since_date_invalid_format_returns_none`, `days_since_date_past_date_is_positive`, `estimate_status_age_from_lifecycle_log`, `estimate_status_age_picks_latest_entry` |

## Coverage Gaps

- Integration gap: add a fixture for "Promote draft to review" before changing user-visible CLI output, generated files, or error handling in cmd_lifecycle.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Promote draft to review | spec `auth` has `status: draft` | `cmd_promote(root, "auth", Text, false)` runs | updates `auth.spec.md` to `status: review` |
| Guard blocks transition | transition guard requires min_score of 60 | spec has score 45 | prints guard failure and exits 1 |
| Status of all specs | multiple specs with various statuses | `cmd_status(root, None, Text)` runs | prints specs grouped by status with colored labels |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec filter matches no specs | Exits 1 with error message | Keep or add a focused assertion before changing this behavior |
| Ambiguous spec filter (multiple matches) | Exits 1, lists all matches | Keep or add a focused assertion before changing this behavior |
| No `status:` line in frontmatter | Prints error, exits 1 | Keep or add a focused assertion before changing this behavior |
| Invalid transition (without `--force`) | Prints error with valid alternatives, exits 1 | Keep or add a focused assertion before changing this behavior |
| Guard check fails (without `--force`) | Prints guard failures, exits 1 | Keep or add a focused assertion before changing this behavior |
| File write fails | Prints error, exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- lifecycle --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/lifecycle.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
