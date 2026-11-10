## ADDED

### REQUIREMENT REQ-importer-002

An `ImportedItem` SHALL carry the content preserved from a local markdown source — its `files` list, declared frontmatter fields, title and existing sections — and `render_spec` SHALL emit the real `files` list and reuse preserved sections instead of always writing a fixed skeleton with `files: []`.

Acceptance Criteria
- `render_spec` writes the item's preserved `files` entries into the frontmatter `files` list verbatim, and only emits `files: []` when the source supplied none (external GitHub, Jira and Confluence imports).
- When the item carries preserved sections, `render_spec` reuses the source's Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies and Change Log bodies, scaffolds only the sections the source lacks, keeps source sections outside the canonical set, and appends the import Change Log row to an existing Change Log instead of replacing it.
- A spec rendered from a frontmatter-complete source passes `specsync check --strict` immediately after import, with no frontmatter failure.
- `ImportSource` names a local directory import as itself; no directory import is reported as Confluence.
