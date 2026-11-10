## ADDED

### REQUIREMENT REQ-cmd-import-002

`import --from-dir` SHALL parse each markdown source's YAML frontmatter and preserve its declared `module`, `version`, `status` and `files` values, SHALL refuse a source whose files list cannot be determined, and SHALL warn whenever a written draft spec still has an empty files list.

Acceptance Criteria
- A source file whose frontmatter declares `module` and a non-empty `files` list imports with those values intact; the module name is not re-derived from the filename.
- Frontmatter-less derivation (title from the first `# ` heading, purpose from the first non-empty paragraph, requirements via `importer::extract_requirements_pub`) remains the fallback when the source declares no frontmatter value for that field.
- A source with no determinable `files` list fails the item with a message naming the missing `files:` frontmatter list, is counted as a batch error, and the batch exits 1.
- Directory imports stamp the Change Log with the local directory source and the source file name, never `Imported from Confluence`.
- Any import verb that writes a spec with an empty files list prints a warning that `specsync check --strict` will fail until `files` is filled in, instead of reporting unqualified success.
