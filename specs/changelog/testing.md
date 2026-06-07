---
spec: changelog.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/changelog.rs` | cargo test changelog:: | `test_parse_range_valid`, `test_parse_range_head_tilde`, `test_parse_range_invalid_no_dots`, `test_parse_range_invalid_empty_from`, `test_parse_range_invalid_empty_to`, `test_parse_range_commit_hashes` |

## Coverage Gaps

- Integration gap: add a fixture for "Generate changelog between two tags" before changing user-visible CLI output, generated files, or error handling in changelog.

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
