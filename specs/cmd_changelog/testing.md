---
spec: cmd_changelog.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/changelog.rs` | cargo test commands::changelog | Command wrapper has no inline tests (range guard + format dispatch only); cover end-to-end before risky changes |
| `src/changelog.rs` range parsing | cargo test changelog::tests::test_parse_range | `test_parse_range_valid`, `test_parse_range_head_tilde`, `test_parse_range_invalid_no_dots`, `test_parse_range_invalid_empty_from`, `test_parse_range_invalid_empty_to`, `test_parse_range_commit_hashes` |
| `src/changelog.rs` generation | cargo test changelog::tests::test_generate_changelog | `test_generate_changelog_no_changes`, `test_generate_changelog_added_spec`, `test_generate_changelog_removed_spec`, `test_generate_changelog_modified_spec` |
| `src/changelog.rs` formatters | cargo test changelog::tests::test_format | `test_format_text_*`, `test_format_json_structure`, `test_format_markdown_empty`, `test_format_markdown_all_sections` |

## Coverage Gaps

- No test drives `cmd_changelog` itself, so the format-dispatch mapping (Text/Github/Table/Csv → format_text) and the invalid-range exit path are unverified at the wrapper level. Add a CLI test before changing dispatch.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Valid range with changes | specs added/modified/removed between two refs | `cmd_changelog(root, "v0.1..v0.2", Text)` | prints added, modified, and removed specs via `format_text` |
| Empty range (no spec changes) | refs with no spec diffs | `cmd_changelog(root, "HEAD~1..HEAD", Json)` | `format_json` emits an empty/zeroed report |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Range missing `..` (or empty side) | Prints "Invalid range format…" and exits 1 | Keep or add a focused assertion before changing this behavior |
| Invalid git refs | git command in the delegate fails; error surfaces | Keep or add a focused assertion before changing this behavior |
| Github/Table/Csv format requested | Falls through to `format_text` | Keep or add a focused assertion before changing this mapping |

## Reviewer Checklist

- Run `cargo run -- changelog --help` and confirm the help text still names the documented flags and behavior.
- Run `cargo test changelog` when changing the delegate; run `cargo test commands::changelog` when changing the wrapper.
- Reproduce one Behavioral Verification row with a temporary git fixture before changing user-visible output.
- If the invalid-range message or a format mapping changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
