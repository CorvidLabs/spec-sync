## MODIFIED

### REQUIREMENT REQ-change-115

The system SHALL accept a `verification_ignored_paths` entry only when it is a bounded, portable, project-relative directory prefix that overlaps no specs, sources, SDD policy inputs or `meaningful_paths` scope, SHALL refuse an invalid list in every lifecycle verb that loads SDD policy, and SHALL refuse, for a change, an entry that overlaps that change's declared `affected_paths`. A path below a `source_dirs` entry is not a source when one of its components below that entry is exactly a `config.toml` `exclude_dirs` name, because the source walk never enters it (#1083).

Acceptance Criteria
- An entry is refused, naming it and the rule, unless it ends in `/`, has no glob or pattern character, is relative with no empty, `.` or `..` component, backslash or control character, is not and does not lie under a symlink on disk, is at most 128 bytes, and neither duplicates nor nests in another entry; the list holds at most 8 entries.
- An entry is refused when it overlaps, ignoring ASCII case, `.specsync/`, the configured specs tree, a `source_dirs` entry, the configured `schema_dir`, any `meaningful_paths` scope, an SDD policy file, the `principles_file`, a `custom_artifacts` template, a file listed in a spec's `files:`, a path-form spec `depends_on` entry, or a file named in a spec's `Consumed By` table; an `ignored_paths` scope covering the entry does not relax this.
- An entry that lies under a `source_dirs` entry is not refused for that overlap when one of its components below the `source_dirs` entry equals an `exclude_dirs` name, compared exactly as the source walk compares names; every other protected scope still applies. With `source_dirs = ["."]` and `exclude_dirs = [".chough"]`, the entries `.chough/sessions/`, `.chough/archive/` and `.chough/bugs/` are accepted, while `.Chough/sessions/` is still refused; with `source_dirs = [".chough/tools"]` and the same `exclude_dirs`, `.chough/tools/cache/` is still refused.
- In a project rooted in a subdirectory of its Git repository, an entry excludes only paths under that project's own copy of the prefix.
- `change new`, `change answer`, `change approve`, `change check`, `change review`, `change finalize`, `change ship` and `change audit` refuse an invalid list as an invalid SDD policy, `change status` makes fixing the entry its one next action, `change ship-status` reports it as a blocker naming the entry, and no evidence is recorded against it.
- A change whose `affected_paths` overlap an entry is refused at create, at the paths answer, and at approve, check, review and finalize; other changes are unaffected.
- `specsync check` does not read the list.
