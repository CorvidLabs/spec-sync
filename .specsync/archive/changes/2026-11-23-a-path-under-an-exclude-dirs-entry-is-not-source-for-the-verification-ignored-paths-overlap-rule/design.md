---
change: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule
artifact: design
---

# Design

- `exclude_dirs` entries are names, not paths. The coverage source walk skips every child whose name equals one exactly (`coverage_name_is_excluded`), and only while descending below a `source_dirs` root.
- So the overlap rule asks one question for the `source_dirs` scope only: does a component of the entry, below the `source_dirs` entry it lies under, equal an `exclude_dirs` name exactly? If so the walk never reaches anything under the entry, and the entry does not overlap that source scope.
- The prefix up to the source root is compared ignoring ASCII case, as the overlap test compares it. The excluded component is compared exactly, as the walk compares it: `.Chough/sessions/` stays refused when `.chough` is excluded.
- An excluded name at or above a `source_dirs` root excludes nothing below it (`source_dirs = [".chough/tools"]` keeps `.chough/tools/cache/` refused), because the walk starts at the root.
- Every other protected scope (`.specsync/`, the specs tree, `schema_dir`, `meaningful_paths`, spec `files:`, `depends_on`, `Consumed By`, the policy files, the principles file and the templates) is unchanged, and the scopes keep their order, so each existing refusal names the same rule.
- `exclude_patterns` is not counted, and `meaningful_paths = ["."]` still blocks the list (the definition's section 6).
