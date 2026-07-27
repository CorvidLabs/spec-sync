---
spec: compact.spec.md
---

## Key Decisions

- **Fence-aware section slicing**: `compact_spec_changelog` recognizes only an exact `## Change Log` H2 outside fenced/indented code, computes the next real H2/EOF boundary, and rewrites only that slice. The rest of the file is copied byte-for-byte.
- **One contiguous validated table**: the first unfenced pipe table is validated against its header width; fenced examples, indented pipe-code, and later tables after prose/blank separation are not changelog data. An indented separator is malformed.
- **Keep-last semantics**: the most recent `keep` rows survive; older rows collapse into one summary row inserted at the position of the first removed row.
- **Provenance-bound summary ownership**: a generated summary carries `<!-- specsync:compact:v1 -->`; unmarked lookalikes remain user history and duplicate marked summaries fail closed.
- **Canonical summary position**: generated summaries must be the first data row; reordered marked state is rejected before reconstruction so EOF-byte preservation cannot concatenate retained rows.
- **Idempotent summary folding**: prior generated counts are accumulated, the original range start is retained, and a no-excess re-run returns the original bytes.
- **Escape/code-span-aware columns**: only odd backslash runs escape pipes, code-span pipes remain cell content, and 3+ column tables get `—` placeholders.
- **Byte-preserving reconstruction**: inclusive source lines retain each untouched LF/CRLF terminator, including mixed-ending files and the no-final-newline state when `keep = 0`.
- **Preflight before publication**: reads, parsing, permissions, and same-directory temporary-file staging complete before any atomic replacement is published; every planned result remains visible on staging failure and the typed report records exact late-publish partial progress.
- **No-op filtering**: `compact_spec_changelog` still returns a `CompactResult` when nothing is removed, but `compact_changelogs` drops those (only `removed > 0` is surfaced and written).

## Files to Read First

- `src/compact.rs` — entire module: `compact_changelogs` (driver), `compact_spec_changelog` (core rewrite), summary metadata/cell parsing helpers, and the `CompactResult` struct.

## Current Status

Issue #417's core behavior includes adversarial ownership, fenced/indented heading and table isolation, canonical summary position, EOF/line-ending fidelity, overflow, complete staging counts, and deterministic partial-publication regressions. Structured rendering is covered in the owning command modules.

## Notes

- Depends on `validator::find_spec_files`.
- Dry-run returns the complete plan without staging or publishing writes.
- Summary rows end with `<!-- specsync:compact:v1 -->` and use `—` cells for wider tables.
- Rewriting preserves every untouched line and its exact terminator.
