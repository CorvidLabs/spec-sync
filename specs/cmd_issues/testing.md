---
spec: cmd_issues.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/issues.rs` | cargo test commands::issues | No inline tests found; add focused coverage for `cmd_issues`, `load_config`, `parse_frontmatter`, `OutputFormat` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "All references valid" before changing user-visible CLI output, generated files, or error handling in cmd_issues.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| All references valid | specs reference issues #10, #15, #20 — all exist and are open | `cmd_issues` runs | prints "3 valid, 0 closed, 0 not found" and exits 0 |
| Stale reference | spec references issue #5 which was deleted | `cmd_issues` runs | prints error for issue #5 and exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| GitHub repo unresolvable | Exits 1 with error message | Keep or add a focused assertion before changing this behavior |
| `gh` CLI not available | API calls fail, counted as errors | Keep or add a focused assertion before changing this behavior |
| Issue returns 404 | Counted as "not found", triggers non-zero exit | Keep or add a focused assertion before changing this behavior |
| API rate limit | Counted as "error", reported but does not halt | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- issues --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/issues.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
