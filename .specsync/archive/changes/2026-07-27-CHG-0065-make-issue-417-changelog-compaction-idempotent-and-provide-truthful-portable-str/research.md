---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: research
---

# Research

## Reproductions

- A second compaction previously consumed or rewrote the tool-generated summary instead of becoming
  a byte-identical no-op.
- Counting the summary as a retained entry made `--keep 5` report six retained rows.
- Naive `|` splitting broke cells containing escaped pipes and could produce the wrong number of
  summary cells for wider tables.
- Reconstructing a section with `lines().join()` removed its trailing newline.
- Root dispatch did not consistently pass the selected output format to compact/archive-tasks.
- Hosted Windows integration tests failed because Markdown rows contained
  `specs\history\history.spec.md` and `specs\work\tasks.md` instead of portable paths.

## Compatibility findings

Normalization belongs at the structured presentation boundary but must be host-aware: `\` is a
separator on Windows and a legal filename byte on Unix. Markdown backslash escaping does not close
code spans, so variable-length delimiters and diagnostic sanitization are required.

Shape alone is not provenance. Generated summaries need an explicit marker, duplicate markers are
ambiguous, counts require checked fixed-width arithmetic, and table parsing must stop after the
first contiguous width-valid table. Inclusive-line reconstruction preserves CRLF/mixed endings.

Repository-wide maintenance writes require plan/stage/publish reporting. All preflight and staging
must finish before mutation; any late publish/rollback failure must stay visible as incomplete or
partial and force exit 1.

No external dependency or protocol research is required. The private sandbox replay will validate
the released binary against a separate repository rather than relying only on in-repository
fixtures.
