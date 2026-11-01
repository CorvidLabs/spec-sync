## ADDED

### REQUIREMENT REQ-change-116

The lesson bundle that finalization writes SHALL add no U+2013 (en dash) or U+2014 (em dash) of its own; user-supplied record text it copies SHALL be copied unchanged.

Acceptance Criteria
- The bundle's first line is `# Lesson bundle: <id>`.
- The bundle's fixed text (heading, section headings, labels and the guidance paragraph) contains neither character.
- User-supplied record text is copied unchanged, including any dash it holds: the change title, every acceptance criterion, affected specs and paths, recorded verification command strings, and the bodies of the change's `context.md`, `design.md` and `testing.md`.
- A bundle assembled from a record and artifacts whose user-supplied text holds neither character contains neither character.
- Archived bundles are not rewritten, and nothing reads the heading back.
