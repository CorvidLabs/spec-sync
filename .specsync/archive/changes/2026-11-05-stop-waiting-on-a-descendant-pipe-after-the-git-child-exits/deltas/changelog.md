## MODIFIED

### SPEC SECTION Invariants

1. Spec comparison uses git `ls-tree` and `show` to read spec state at each ref — never touches the working tree
2. Field-level diffing compares frontmatter fields: module, version, status, files, db_tables, depends_on
3. `parse_range` only accepts `..` as separator and requires non-empty parts on both sides
4. Added/removed/modified lists are sorted by spec path (BTreeMap/BTreeSet ordering)
5. A spec is "modified" only if at least one frontmatter field changed between the two refs
6. Output formatters (text, json, markdown) share the same ChangelogReport — formatting is decoupled from analysis
7. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
