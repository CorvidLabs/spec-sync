---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: docs
---

# Docs

Update canonical specs and companions for `archive`, `compact`, `cmd_compact`,
`cmd_archive_tasks`, and `cli`.
The documentation must state:

- provenance-bound ownership and ambiguity/overflow rejection for generated summary rows;
- idempotent folding, pipe-escape/code-span handling, table isolation, exact LF/CRLF behavior,
  and truthful retained counts;
- supported text, JSON, Markdown, and GitHub maintenance output;
- equivalence of `--json` and `--format json`;
- dry-run `would_change`/`applied` semantics; and
- platform-aware separators that preserve Unix literal backslashes;
- injection-safe Markdown/GitHub path rendering; and
- preflight/atomic publication with truthful complete/partial failures and exit status.

Increment affected spec versions and add dated change-log entries. Release or PR notes must call
out the structured-output compatibility correction without claiming a change to text-path spelling.
