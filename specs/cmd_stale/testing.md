---
spec: cmd_stale.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/stale.rs` | cargo test commands::stale | No inline `#[cfg(test)]` module; CLI behavior is covered by the integration tests below |
| `tests/integration.rs` | cargo test --test integration stale_outside_git_repo_fails_with_message | Non-git root: failure exit, stderr contains "Not a git repository" |
| `tests/integration.rs` | cargo test --test integration stale_outside_git_repo_json_reports_error | Non-git root, `--format json`: failure exit, stdout has `"not a git repository"` and `"stale_specs"` |
| `tests/integration.rs` | cargo test --test integration stale_in_fresh_repo_reports_all_up_to_date | Repo where spec and source share history: success exit, stdout contains "up to date" |
| `src/git_utils.rs` | cargo test git_utils | Underlying commit-distance logic (`commits_since_counts_source_changes_after_spec`, etc.) lives in git_utils tests |

## Coverage Gaps

- No integration fixture yet exercises a *stale* spec end-to-end (a committed spec followed by N commits to its source). The git-distance counting is covered at the unit level in `src/git_utils.rs`.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| All specs fresh | all specs were updated after their source files | `specsync stale` is run | prints "All specs are up to date" and exits 0 |
| Spec behind source by 8 commits (threshold 5) | module "auth" has source file `src/auth.rs` with 8 commits since spec was last updated | `specsync stale --threshold 5` is run | reports auth as stale with "8 commits behind" and exits 1 |
| JSON output | 2 stale specs out of 10 total | `specsync stale --format json` is run | outputs JSON with `total_specs: 10`, `stale_count: 2`, `stale_specs` array with per-file details |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Not a git repository | Prints error, exits 1 | Covered by `stale_outside_git_repo_fails_with_message` / `stale_outside_git_repo_json_reports_error` |
| Spec file unreadable | Skipped silently | Keep or add a focused assertion before changing this behavior |
| No frontmatter | Skipped silently | Keep or add a focused assertion before changing this behavior |
| Source file doesn't exist on disk | Skipped in commit distance check | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- stale --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/stale.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
