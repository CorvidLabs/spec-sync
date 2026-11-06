## MODIFIED

### SPEC SECTION Invariants

1. When `repo` and `branch` are provided, spec links are full GitHub URLs (`https://github.com/{repo}/blob/{branch}/{path}`); otherwise relative markdown links
2. Error messages are classified into actionable suggestion categories: missing sections, missing source files, DB table issues, frontmatter problems, dependency issues
3. The comment header includes a pass/fail icon and status based on `overall_passed`
4. Coverage metrics (file and LOC percentages) are always included in the summary table
5. `detect_branch` returns `None` if not in a git repository or git command fails
6. The marketplace GitHub Action (`action.yml`) and project CI workflow (`.github/workflows/ci.yml`) both invoke `specsync comment` to produce identical PR comment output
7. Rendered bodies are at most 49,152 bytes, are always valid UTF-8, and include local reproduction guidance when truncated
8. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
