# Lesson bundle: import-from-dir-preserves-source-content

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Import --from-dir preserves source content
- **Kind**: BugFix
- **Specs**: importer, cmd_import
- **Paths**: src/importer.rs, src/commands/import.rs, specs/importer/, specs/cmd_import/, tests/, site/src/content/docs/cli.md
- **Acceptance**: import --from-dir keeps the source file's module name, files list, purpose and existing sections (Public API, Invariants, Change Log); the imported spec passes specsync check --strict immediately; the Change Log names the real source instead of Confluence; a source with no usable files list is refused loudly and counted as an error

## Evidence

- Verification commit: `8045cdacb66b70953bef8e2dd460b645d4621a43`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec cmd_import --spec importer`

## From the change's context.md

# Context

GitHub issue #416: `import --from-dir` discards source content, writes a skeleton with
hardcoded `files: []` that fails the tool's own gate, and stamps `Imported from Confluence:`
on directory imports. Verified still reproducing on main 49d2326a (specsync 6.2.0).

Root causes: `parse_markdown_as_import_item` never strips YAML frontmatter;
`importer::render_spec` hardcodes `files: []` (shared by all import verbs);
`ImportedItem` has no `files` field; directory items are mislabeled
`ImportSource::Confluence`.

External importers (GitHub/Jira/Confluence) have no source file list, so they keep an
empty `files:` and the command warns that the draft needs `files` before the gate passes,
instead of printing a clean success.

## From the change's testing.md

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

## Where these lessons go

- `specs/importer/importer.3md`
- `specs/cmd_import/cmd_import.3md`
