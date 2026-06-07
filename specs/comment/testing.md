---
spec: comment.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/comment.rs` | cargo test comment:: | `test_spec_link_with_repo`, `test_spec_link_without_repo`, `test_suggestion_for_missing_section`, `test_suggestion_for_source_file_not_found`, `test_suggestion_for_db_table`, `test_suggestion_for_frontmatter` |

## Coverage Gaps

- Integration gap: add a fixture for "Render passing check comment" before changing user-visible CLI output, generated files, or error handling in comment.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Render passing check comment | 10 specs checked, all passed, 0 errors, 0 warnings | `render_check_comment(10, 10, 0, 0, &[], &[], &coverage, true, Some("org/repo"), Some("main"))` is called | returns markdown with "✅ SpecSync: Passed" header and summary table |
| Render failing check comment with errors | 10 specs checked, 8 passed, 2 with errors pointing to `specs/auth/auth.spec.md` and repo "org/repo" on branch "feat/auth" | `render_check_comment` is called with errors | error lines include clickable GitHub links to the spec file |
| Detect branch | a git repository on branch `feat/new-module` | `detect_branch(root)` is called | returns `Some("feat/new-module")` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Not in a git repository | `detect_branch` returns `None` | Keep or add a focused assertion before changing this behavior |
| No repo/branch provided | Spec links use relative markdown format instead of GitHub URLs | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/comment.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
