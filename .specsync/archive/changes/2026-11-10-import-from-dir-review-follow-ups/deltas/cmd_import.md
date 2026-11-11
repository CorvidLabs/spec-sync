## MODIFIED

### REQUIREMENT REQ-cmd-import-002

`import --from-dir` SHALL parse each markdown source's YAML frontmatter and preserve its declared `module`, `version` and list fields, SHALL import a parseable markdown doc without frontmatter as a draft with a loud empty-files warning, and SHALL warn whenever a written draft spec still has an empty files list.

Acceptance Criteria
- A source file whose frontmatter declares `module` and a non-empty `files` list imports with those values intact; the module name is not re-derived from the filename.
- Frontmatter-less derivation (title from the first `# ` heading, purpose from the first non-empty paragraph, requirements via `importer::extract_requirements_pub`) remains the fallback when the source declares no frontmatter value for that field.
- A parseable source with no determinable `files` list imports as a `status: draft` spec and the command prints a warning that `specsync check --strict` will fail until `files` is filled in; only unreadable or unparseable sources count as batch errors with exit 1.
- Directory imports stamp the Change Log with the local directory source and the source file name, never `Imported from Confluence`.
- Any import verb that writes a spec with an empty files list prints the empty-files warning instead of reporting unqualified success.
