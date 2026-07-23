## ADDED

### REQUIREMENT REQ-registry-003

Local and remote registries SHALL be parsed as TOML with explicit schema validation and confined
module mappings.

Acceptance Criteria

- The parser accepts canonical `[registry]` plus `[specs]` mappings.
- The parser also accepts `[registry]` plus documented `[[modules]]` records in which every record
  has a string `name` and string `spec` path.
- Generation, initialization, and registration continue to emit one deterministic `[specs]` table
  sorted by module name; `[specs]` is canonical and `[[modules]]` is accepted compatibility input.
- Malformed TOML, wrong field types, incomplete module entries, duplicate module identities,
  conflicting mappings across accepted shapes, and non-string mappings fail closed.
- Every mapped spec path is a portable project-relative path confined beneath the project root;
  absolute, traversal, backslash, drive/UNC, and symlink-escaping mappings fail before file access.
- A valid legacy stub with no registry name and no authoritative mappings remains `Ok(None)`;
  malformed TOML is never classified as inert.
- Local parse failure includes `failed to parse local registry <path>` and preserves the dependency
  resolution context that identifies the triggering declaration.
- Registry-backed non-conventional locations resolve successfully and are not subjected to
  conventional directory-name identity rules.

## MODIFIED

### SPEC SECTION Invariants

1. Registry bytes are parsed as TOML with explicit schema validation; line scanning is not an
   authoritative parser.
2. Accepted input is `[registry]` with canonical `[specs]` mappings or compatible `[[modules]]`
   records containing string `name` and `spec` fields.
3. Malformed TOML, wrong field types, incomplete records, duplicate identities, conflicting
   mappings, and non-string mappings fail closed.
4. Registry mappings are portable project-relative spec paths and are confined lexically and
   through symlink or nearest-existing-ancestor checks before file, request, or cache access.
5. A syntactically valid registry with no name and no authoritative mappings is the only inert
   legacy-stub case; malformed content is never inert.
6. Local registry resolution prefers `.specsync/registry.toml`; the legacy root file is used only
   for an unmigrated local layout.
7. Generated and registered output uses one deterministic `[specs]` table sorted by module name
   and safely serializes TOML keys and values.
8. Registry-backed custom spec locations derive module identity from the registry key rather than
   a conventional directory name.
9. Remote registries use the shared authenticated GitHub content transport and the same TOML parser
   and mapping validation as local registries.
