## ADDED

### REQUIREMENT REQ-generator-009

Spec creation source selection SHALL exclude any file already listed in an existing spec's frontmatter `files` list, so `new`, `scaffold`, and generated specs never claim a file another spec owns.

Acceptance Criteria
- Candidate collection in `find_creation_module_sources` and the single-source fallback filter out owned files before a spec is written.
- A module whose candidates are all owned produces no spec (the caller's existing no-files path).
- Explicit `config.modules` mappings are not filtered — the config is the author's explicit ownership declaration.
