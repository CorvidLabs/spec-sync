---
spec: merge.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/merge.rs` | cargo test merge:: | `test_has_conflict_markers`, `test_parse_conflict_regions`, `test_resolve_changelog_conflict`, `test_resolve_table_conflict`, `test_resolve_frontmatter_conflict`, `test_full_spec_conflict_resolution` |

## Coverage Gaps

- Integration gap: add a fixture for "Auto-resolve frontmatter list conflict" before changing user-visible CLI output, generated files, or error handling in merge.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Auto-resolve frontmatter list conflict | ours has `files: [a.rs, b.rs]` and theirs has `files: [b.rs, c.rs]` | `merge_specs` resolves the conflict | result is `files: [a.rs, b.rs, c.rs]` (union, sorted) |
| Auto-resolve changelog conflict | ours added a `2026-03-20` entry, theirs added a `2026-03-25` entry | `merge_specs` resolves the conflict | both entries appear in chronological order, status is `Resolved` |
| Prose conflict requires manual resolution | both sides modified the `## Purpose` section text | `merge_specs` encounters the conflict | conflict markers are preserved, status is `Manual` |
| Dry run | conflicted spec files exist | `merge_specs(root, specs_dir, true, false)` is called | returns `MergeResult` entries with resolution details but does not modify files |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Marked as `Manual` with read error in details | Keep or add a focused assertion before changing this behavior |
| `git diff` command fails | Falls back to scanning all files for conflict markers | Keep or add a focused assertion before changing this behavior |
| Post-resolution frontmatter invalid | Warning printed; file is still written with resolved content | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/merge.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
