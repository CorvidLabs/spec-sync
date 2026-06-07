---
spec: importer.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/importer.rs` | cargo test importer:: | `test_slugify_simple`, `test_slugify_special_chars`, `test_slugify_already_slug`, `test_slugify_mixed_case_spaces`, `test_slugify_empty`, `test_extract_requirements_checkboxes` |

## Coverage Gaps

- Integration gap: add a fixture for "Import GitHub issue with acceptance criteria" before changing user-visible CLI output, generated files, or error handling in importer.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Import GitHub issue with acceptance criteria | GitHub issue #42 titled "Add user auth" with body containing checkboxes | `import_github_issue("org/repo", 42)` is called | returns `ImportedItem` with `module_name: "add-user-auth"`, `issue_number: Some(42)`, and extracted requirements from checkboxes |
| Import Jira issue with ADF description | Jira issue `PROJ-123` with ADF-format description containing acceptance criteria | `import_jira_issue("PROJ-123")` is called | extracts text from ADF content tree and parses requirements |
| Import Confluence page | Confluence page ID `98765` with HTML storage body | `import_confluence_page("98765")` is called | strips HTML, extracts purpose from first line, and parses requirements |
| Render spec with issue number | an `ImportedItem` with `issue_number: Some(42)` | `render_spec(&item)` is called | generated frontmatter contains `implements: [42]` |
| Render spec without issue number | an `ImportedItem` with `issue_number: None` (Jira/Confluence) | `render_spec(&item)` is called | generated frontmatter contains `implements: []` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `JIRA_URL` not set | `import_jira_issue` returns `Err("JIRA_URL environment variable not set")` | Keep or add a focused assertion before changing this behavior |
| `JIRA_TOKEN` not set | `import_jira_issue` returns `Err("JIRA_TOKEN environment variable not set")` | Keep or add a focused assertion before changing this behavior |
| `CONFLUENCE_URL` not set | `import_confluence_page` returns `Err("CONFLUENCE_URL environment variable not set")` | Keep or add a focused assertion before changing this behavior |
| `CONFLUENCE_TOKEN` not set | `import_confluence_page` returns `Err("CONFLUENCE_TOKEN environment variable not set")` | Keep or add a focused assertion before changing this behavior |
| GitHub: neither `gh` nor `GITHUB_TOKEN` | `import_github_issue` returns `Err` | Keep or add a focused assertion before changing this behavior |
| Issue/page not found (404) | Each importer returns `Err("{type} not found")` | Keep or add a focused assertion before changing this behavior |
| Network timeout | Returns `Err` with connection details | Keep or add a focused assertion before changing this behavior |
| Invalid issue number for GitHub | CLI rejects before calling importer | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/importer.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
