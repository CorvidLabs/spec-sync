## MODIFIED

### SPEC SECTION Invariants

1. Uses `git diff --name-only {base}` to get the list of changed files
2. Only files matching `config.source_extensions` are considered
3. Export deltas compare each affected spec's currently-exported symbols (parsed from the working-tree source files) against the symbols it documents: `new_exports` are current exports the spec does not document, `removed_exports` are documented symbols no longer exported (there is no base-ref content fetch)
4. Changed files not covered by any spec are listed separately
5. Text output delegates to `output::print_diff_markdown`
6. A non-zero `git diff` exit (bad base ref, not a git repository) fails loud with exit code 1 — it is never treated as an empty change set ("No files changed"), so a failed comparison cannot silently pass in CI
7. A spec whose `files:` includes a source file that exists-or-should but cannot be read (missing, or not valid UTF-8) is inconclusive: the file is surfaced (`inconclusive_files`), its unreliable export deltas are suppressed, and `diff` exits 1 — an unreadable source must not be silently treated as export-free "no drift" (a non-source file, e.g. a `.md`/`.sql` listed in `files:`, is not a failure)
8. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
