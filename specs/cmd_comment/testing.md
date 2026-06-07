---
spec: cmd_comment.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/comment.rs` | cargo test commands::comment | No inline tests found; add focused coverage for `cmd_comment`, `build_comment_body`, `resolve_repo` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Print to stdout" before changing user-visible CLI output, generated files, or error handling in cmd_comment.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Print to stdout | `--pr` is not set | `cmd_comment` runs | prints markdown summary to stdout |
| Post to PR | `--pr 42` is set | `cmd_comment` runs | posts comment on PR #42 |
| Marketplace action captures stdout | the marketplace action runs with `comment: true` | `specsync comment` is invoked without `--pr` | the stdout output is identical to what the CI workflow captures via `cargo run -- comment` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `gh` CLI not installed | Command fails with error | Keep or add a focused assertion before changing this behavior |
| GitHub repo unresolvable | Exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- comment --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/comment.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
