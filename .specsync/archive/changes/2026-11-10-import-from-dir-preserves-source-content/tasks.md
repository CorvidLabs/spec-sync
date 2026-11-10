---
change: import-from-dir-preserves-source-content
artifact: tasks
---

# Tasks

- [x] Parse YAML frontmatter in `parse_markdown_as_import_item` (module, version, status, files) instead of treating it as body text
- [x] Add `files: Vec<String>` to `ImportedItem`; `render_spec` emits the real list instead of hardcoded `files: []`
- [x] Preserve existing source sections (Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log) when rendering; scaffold only missing sections
- [x] Stop mapping directory imports to `ImportSource::Confluence`; name the real source in the Change Log
- [x] Refuse a source whose `files` list cannot be determined, loudly, counted as a batch error
- [x] Warn on import when the written spec will fail `check --strict` (empty files), for all import verbs
- [x] Update `specs/importer/` and `specs/cmd_import/` (spec, requirements, 3md lesson) and add regression tests
