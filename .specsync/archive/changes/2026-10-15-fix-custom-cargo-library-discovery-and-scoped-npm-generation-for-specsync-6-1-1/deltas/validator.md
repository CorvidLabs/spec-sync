## ADDED

### REQUIREMENT REQ-validator-044

Coverage SHALL admit an exact project-root source file only when checked Cargo discovery identifies that declared target and the selected source list includes it. The file SHALL be read through the retained project capability with existing file-identity, confinement and resource protections, without recursively measuring its parent.

Acceptance Criteria
- Exact-file admission is limited to a declared root-level Cargo target; arbitrary file entries do not broaden source scope.
- No-follow regular-file checks reject links, directories and special files; expected identity, UTF-8 validation and existing file/entry/byte bounds remain enforced.
- Existing extension/test filters apply, and overlapping selections do not count the file twice.
- Explicit configured directory scope remains authoritative; discovery does not append targets outside it.
- Coverage and generated mappings include the selected declared target without unrelated root files.
