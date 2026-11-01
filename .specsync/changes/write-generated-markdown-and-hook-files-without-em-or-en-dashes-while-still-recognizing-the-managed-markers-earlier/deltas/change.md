## ADDED

### REQUIREMENT REQ-change-113

The lesson bundle that finalization writes SHALL add no U+2013 (en dash) or U+2014 (em dash) of its own; text it copies from the change's artifacts SHALL be copied unchanged.

Acceptance Criteria
- The bundle's first line is `# Lesson bundle: <id>`.
- A bundle assembled from artifacts that contain neither character contains neither character.
- A dash in the change's `context.md`, `design.md` or `testing.md` appears unchanged in the bundle.
- Archived bundles are not rewritten, and nothing reads the heading back.
