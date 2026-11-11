---
change: import-from-dir-preserves-source-content
artifact: context
---

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
