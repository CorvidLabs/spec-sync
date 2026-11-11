---
change: import-from-dir-review-follow-ups
artifact: testing
---

# Testing


Unit: fenced `##` stays in its section; depends_on/db_tables/implements/lead-in preserved;
status: active source imports as draft with a Change Log note. Integration: frontmatter-less
doc imports with warning and exit 0; imported spec has no frontmatter failure under `check`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-importer-002 | unit tests `test_import_local_markdown_fence_headings_stay_content`, `test_import_local_markdown_preserves_lists_and_lead_in`, `test_import_local_markdown_preserves_frontmatter_and_sections`, `test_import_local_markdown_appends_to_existing_change_log`, `test_import_local_markdown_keeps_unknown_sections`; integration `import_from_dir_imports_markdown_files` (asserts check has no frontmatter failure) |
| REQ-cmd-import-002 | integration `import_from_dir_warns_for_doc_without_files` (exit 0, draft written, warning printed); unit `test_import_local_markdown_without_files_imports_as_warned_draft` |
