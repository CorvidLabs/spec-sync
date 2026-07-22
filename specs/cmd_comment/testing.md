---
spec: cmd_comment.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/comment.rs` | cargo test commands::comment | Command wrapper has no inline tests (pipeline reuse + gh shell-out branch); cover stdout mode end-to-end before risky changes |
| `src/comment.rs` rendering | cargo test comment::tests::test_render_check_comment | `test_render_check_comment_passed`, `test_render_check_comment_failed_with_errors`, `test_render_check_comment_has_footer`, `test_render_check_comment_truncates_unspecced_files` |
| `src/comment.rs` suggestions | cargo test comment::tests::test_suggestion | `test_suggestion_for_missing_section`, `test_suggestion_for_source_file_not_found`, `test_suggestion_for_db_table`, `test_suggestion_for_dependency`, … |
| `src/comment.rs` grouping/links | cargo test comment::tests | `test_group_by_spec`, `test_split_spec_prefix`, `test_strip_spec_prefix`, `test_spec_link_with_repo`, `test_spec_link_without_repo` |
| `tests/integration/commands.rs` | cargo test --test integration malformed_gradle_is_inconclusive_for_coverage_gating_commands | Checked manifest failure exits 1 before markdown or GitHub posting and identifies Gradle on stderr |

## Coverage Gaps

- Healthy stdout mode and the `--pr` posting branch remain unverified at the wrapper level; malformed-discovery fail-closed behavior is covered end to end. Body content is covered through `comment::render_check_comment`.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Print to stdout | `--pr` is not set | `cmd_comment` runs | prints the `render_check_comment` markdown body to stdout, no `gh` invocation |
| Post to PR | `--pr 42` is set, repo resolvable | `cmd_comment` runs | resolves repo and runs `gh pr comment 42 --repo … --body …`, prints "Posted spec-sync comment on PR #42" |
| Status matches `check` | same project, same flags | `cmd_comment` vs `cmd_check` | the comment's pass/fail badge matches `check`'s exit code (both go through `compute_exit_code`) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `gh` CLI not installed | Prints "Failed to run gh CLI" + install hint, exits 1 | Keep or add a focused assertion before changing this behavior |
| `gh pr comment` exits non-zero | Prints the exit code and exits 1 | Keep or add a focused assertion before changing this behavior |
| GitHub repo unresolvable (`--pr` set) | Prints resolver error and exits 1 | Keep or add a focused assertion before changing this behavior |
| Malformed Gradle settings | Prints an inconclusive diagnostic and exits 1 before rendering or posting | Covered by `malformed_gradle_is_inconclusive_for_coverage_gating_commands` |

## Reviewer Checklist

- Run `cargo run -- comment --help` and confirm the help text still names the documented flags (`--pr`, `--strict`, `--enforcement`, `--require-coverage`).
- Run `cargo test comment` when changing the renderer; run `cargo test commands::comment` when changing the wrapper.
- Reproduce the stdout-mode Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
