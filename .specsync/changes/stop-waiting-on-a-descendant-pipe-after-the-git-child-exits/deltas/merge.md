## MODIFIED

### SPEC SECTION Invariants

1. `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
2. `all_files: true` scans all spec files for conflict markers regardless of git state
3. Frontmatter list fields are unioned (not replaced) and sorted alphabetically
4. Numeric frontmatter `version` values use `max(ours, theirs)`; nonnumeric versions and divergent or one-sided other scalars require manual resolution
5. Changelog rows are sorted by first cell (ISO date) — lexicographic sorting works correctly
6. Prose conflicts are never auto-resolved — always marked as `Manual`
7. Every post-resolution candidate must contain frontmatter with valid YAML, no duplicate keys, all required fields, a valid status, and non-empty files; failures are `Manual` and are not written
8. `dry_run: true` returns results without writing resolved content to disk
9. Custom YAML parser handles simple key-value and list fields without external YAML library
10. Only exact standard marker forms are accepted; diff3 base sections are parsed but never treated as either branch's content
11. A file is written only when every conflict hunk is safely resolved and the resulting frontmatter is valid
12. Resolution details name both conflict-marker labels and the applied strategy or manual-resolution reason; candidate hunks say `Auto-resolvable` until a file is actually persisted
13. Orphan, duplicate, nested, incomplete, and malformed marker families make the complete file manual
14. Table headers/separators, including a conflicted header immediately followed by a clean separator, and nested frontmatter mappings are never reconstructed by the lossy subset parsers
15. Uniform CRLF/LF form and final-newline presence are preserved
16. `all_files: false` with no answer from git yields a single `Unknown` result naming the missing precondition and exits non-zero; an unperformed scan is never reported as "no conflicts found"
17. `unmerged_paths` returns `None` — not an empty set — whenever git could not answer, so no caller can read an unasked question as an all-clear
18. `unmerged_paths` spawns git through `crate::git_utils::git_cmd`, so host secrets and git-override variables are not forwarded
19. `conflict_hunks` and `document_conflict_hunks` return only complete opener/separator/closer triples, and `document_conflict_hunks` ignores fenced code blocks, so marker-shaped prose (a setext `<h1>` underline, a documented example) is never reported as a conflict
20. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
