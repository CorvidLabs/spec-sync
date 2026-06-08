---
spec: changelog.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `parse_range` | cargo test changelog::tests::test_parse_range | `test_parse_range_valid`, `test_parse_range_head_tilde`, `test_parse_range_invalid_no_dots`, `test_parse_range_invalid_empty_from`, `test_parse_range_invalid_empty_to`, `test_parse_range_commit_hashes` |
| frontmatter diffing | cargo test changelog::tests::test_compare_frontmatter | `test_compare_frontmatter_no_changes`, `test_compare_frontmatter_status_change`, `test_compare_frontmatter_version_change`, `test_compare_frontmatter_files_change`, `test_compare_frontmatter_depends_on_change`, `test_compare_frontmatter_multiple_changes`, `test_compare_frontmatter_implements_change`, `test_compare_frontmatter_agent_policy_change` |
| section diffing | cargo test changelog::tests::test_compare_sections | `test_compare_sections_no_changes`, `test_compare_sections_modified`, `test_compare_sections_added`, `test_compare_sections_removed`, `test_extract_sections_basic`, `test_extract_sections_ignores_subsections` |
| git-backed generation | cargo test changelog::tests::test_generate_changelog | `test_generate_changelog_no_changes`, `test_generate_changelog_added_spec`, `test_generate_changelog_removed_spec`, `test_generate_changelog_modified_spec` (use a real temp git repo via `setup_git_repo`) |
| formatters | cargo test changelog::tests::test_format | `test_format_text_empty`, `test_format_text_added`, `test_format_text_modified_with_section_changes`, `test_format_json_structure`, `test_format_markdown_empty`, `test_format_markdown_all_sections` |

## Coverage Gaps

- No CLI-level (`tests/integration.rs`) coverage for `specsync changelog <range>`; the `generate_changelog` git path is covered by the in-module `setup_git_repo` tests. Add a CLI fixture before changing the changelog subcommand's terminal output.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Generate changelog between two tags | specs at `v1.0.0` and `v1.1.0` where `auth.spec.md` was added and `config.spec.md` had its version bumped | `generate_changelog(root, "specs", "v1.0.0", "v1.1.0")` is called | returns a report with auth in `added` and config in `modified` with a version FieldChange |
| Parse valid range | the string "v1.0..v2.0" | `parse_range("v1.0..v2.0")` is called | returns `Some(("v1.0", "v2.0"))` |
| Parse invalid range | the string "v1.0" | `parse_range("v1.0")` is called | returns `None` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Git ref doesn't exist | `list_specs_at_ref` returns empty list — no crash | Keep or add a focused assertion before changing this behavior |
| Range string has no `..` separator | `parse_range` returns `None` | Keep or add a focused assertion before changing this behavior |
| Spec frontmatter unparseable at a ref | Spec is silently skipped in diff | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/changelog.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
