## MODIFIED

### REQUIREMENT REQ-importer-002

An `ImportedItem` SHALL carry the content preserved from a local markdown source — its `files`, `depends_on`, `db_tables` and `implements` lists, declared `version`, title, H1 lead-in and existing sections — and `render_spec` SHALL emit the real lists and reuse preserved sections instead of always writing a fixed skeleton with `files: []`. Rendered imports are always `status: draft`.

Acceptance Criteria
- `render_spec` writes the item's preserved frontmatter lists into the frontmatter verbatim, and only emits empty lists when the source supplied none (external GitHub, Jira and Confluence imports).
- When the item carries preserved sections, `render_spec` reuses the source's Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies and Change Log bodies, scaffolds only the sections the source lacks, keeps source sections outside the canonical set, keeps the H1 lead-in paragraph, and appends the import Change Log row to an existing Change Log instead of replacing it.
- Section splitting never treats a `## ` line inside a fenced code block as a heading.
- A spec rendered from a frontmatter-complete source has no frontmatter failure under `specsync check`; `check --strict` behaves exactly as it does for a `specsync new` scaffold (companion scaffold markers remain to be filled).
- Rendered imports always carry `status: draft`; a source's non-draft status is noted in the import Change Log row.
- `ImportSource` names a local directory import as itself; no directory import is reported as Confluence.
