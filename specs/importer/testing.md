---
spec: importer.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/importer.rs` (slugify) | cargo test importer:: | `test_slugify_simple`, `test_slugify_special_chars`, `test_slugify_already_slug`, `test_slugify_mixed_case_spaces`, `test_slugify_empty` |
| `src/importer.rs` (requirement extraction) | cargo test importer:: | `test_extract_requirements_checkboxes`, `test_extract_requirements_criteria_section`, `test_extract_requirements_definition_of_done`, `test_extract_requirements_empty_body` |
| `src/importer.rs` (parsers) | cargo test importer:: | `test_import_github_issue_entry_path_converts_shared_typed_details`, `test_import_github_issue_entry_path_returns_no_item_on_provider_failure`, `test_import_github_issue_details_full`, `test_import_github_issue_details_empty_body`, `test_parse_jira_json_plain_description`, `test_parse_jira_json_adf_description`, `test_parse_confluence_json` |
| Portable provider slugs | cargo test importer::tests::test_external_imports_reject_nonportable_slugs | GitHub, Jira, and Confluence reserved-device titles fail before an item exists; shared validator boundary tests cover overlong ASCII/multibyte candidates |
| GitHub provider boundary | cargo test github::tests | All-platform source guard forbids `gh` construction in importer/read modules; Unix token-present importer entry also fails through an isolated unreachable local REST endpoint without executing a PATH-injected sentinel |
| GitHub CLI failure boundary | cargo test --test integration github_import_fails_closed | Single and batch commands require `GITHUB_TOKEN`, exit non-zero, and create no spec output |
| `src/importer.rs` (render/encode) | cargo test importer:: | `test_render_spec_with_issue_number`, `test_render_spec_without_issue_number`, `test_base64_encode`, `test_strip_html_nested` |

## Coverage Gaps

- Live-success integration gap: add a credential-safe recorded REST fixture for "Import GitHub
  issue with acceptance criteria" before changing successful user-visible output or generated files.

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
| GitHub: `GITHUB_TOKEN` missing or issue 404 becomes repository-inaccessible | Typed shared REST path returns `Err`; no `gh` process is launched and no spec is written | Covered by GitHub provider/token/revalidation tests, importer entry-path tests, and single/batch command regressions |
| Issue/page not found (404) | Each importer returns `Err("{type} not found")` | Keep or add a focused assertion before changing this behavior |
| Network timeout | Returns `Err` with connection details | Keep or add a focused assertion before changing this behavior |
| Invalid issue number for GitHub | CLI rejects before calling importer | Keep or add a focused assertion before changing this behavior |
| Provider title slugifies to an empty or non-portable name | Import returns `Err` before an item or output path exists | Covered by `test_import_github_issue_rejects_title_without_a_safe_module_name`, `test_external_imports_reject_nonportable_slugs`, and shared byte-boundary tests |
| Auth token echoed in a REST error | `redact_secret` replaces the verbatim token with `[REDACTED]` before the `Err` is surfaced | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/importer.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
