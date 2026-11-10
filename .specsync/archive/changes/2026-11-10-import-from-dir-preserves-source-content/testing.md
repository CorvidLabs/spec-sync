---
change: import-from-dir-preserves-source-content
artifact: testing
---

# Testing

- Unit: `parse_markdown_as_import_item` preserves frontmatter module/files and Purpose body; refuses a file with no files list.
- Unit: `render_spec` emits `files: [..]` from the item and keeps source sections.
- Integration: import a frontmatter'd spec from a temp dir; assert purpose text present, module name kept, `check --strict` passes; assert the Change Log does not mention Confluence.
- Integration: a markdown file without frontmatter files is counted as an error and the batch exits 1.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-importer-002 | unit tests `test_import_local_markdown_preserves_frontmatter_and_sections`, `test_import_local_markdown_appends_to_existing_change_log`, `test_import_local_markdown_keeps_unknown_sections`, `test_import_source_display`; integration `import_from_dir_preserves_existing_sections` |
| REQ-cmd-import-002 | integration `import_from_dir_imports_markdown_files` (frontmatter preserved, no Confluence stamp), `import_from_dir_refuses_doc_without_files` (exit 1, nothing written), unit `test_import_local_markdown_refuses_missing_files`, `test_import_local_markdown_refuses_empty_files_list` |
