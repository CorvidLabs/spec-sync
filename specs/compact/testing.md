---
spec: compact.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/compact.rs` | cargo test compact:: | `test_compact_changelog`, `test_compact_no_change_needed`, `test_compact_three_column_table` |

## Coverage Gaps

- Integration gap: add a fixture for "Short changelog (no compaction needed)" before changing user-visible CLI output, generated files, or error handling in compact.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Compact a long changelog | a spec with 20 changelog entries and `keep = 5` | `compact_changelogs` is called | the first 15 entries are replaced with a single summary row of the form `\| <first_date> — <last_date> \| Compacted: 15 entries \|` |
| Short changelog (no compaction needed) | a spec with 3 changelog entries and `keep = 5` | `compact_changelogs` is called | the spec is skipped (not included in results) |
| Dry run | specs with long changelogs | `compact_changelogs(root, specs_dir, 5, true)` is called | returns `CompactResult` entries but does not modify any files |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Prints error in bold red, continues processing other files | Keep or add a focused assertion before changing this behavior |
| No changelog section found | Spec is silently skipped | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/compact.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
