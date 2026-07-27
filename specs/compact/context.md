---
spec: compact.spec.md
---

## Key Decisions

- **String slicing, not markdown parsing**: `compact_spec_changelog` finds the `## Change Log` marker, computes the section end (next `## ` heading or EOF), and rewrites only that slice. The rest of the file is copied byte-for-byte.
- **One contiguous validated table**: the first pipe table is validated against its header width; a later table after prose/blank separation is not changelog data.
- **Keep-last semantics**: the most recent `keep` rows survive; older rows collapse into one summary row inserted at the position of the first removed row.
- **Provenance-bound summary ownership**: a generated summary carries `<!-- specsync:compact:v1 -->`; unmarked lookalikes remain user history and duplicate marked summaries fail closed.
- **Idempotent summary folding**: prior generated counts are accumulated, the original range start is retained, and a no-excess re-run returns the original bytes.
- **Escape/code-span-aware columns**: only odd backslash runs escape pipes, code-span pipes remain cell content, and 3+ column tables get `—` placeholders.
- **Byte-preserving reconstruction**: inclusive source lines retain each untouched LF/CRLF terminator, including mixed-ending files.
- **Preflight before publication**: reads, parsing, permissions, and same-directory temporary-file staging complete before any atomic replacement is published; the typed report carries incomplete/partial failures.
- **No-op filtering**: `compact_spec_changelog` still returns a `CompactResult` when nothing is removed, but `compact_changelogs` drops those (only `removed > 0` is surfaced and written).

## Files to Read First

- `src/compact.rs` — entire module: `compact_changelogs` (driver), `compact_spec_changelog` (core rewrite), summary metadata/cell parsing helpers, and the `CompactResult` struct.

## Current Status

Issue #417's core behavior includes adversarial ownership, parsing, line-ending, overflow, and atomic-write regressions. Structured rendering is covered in the owning command modules.

## Notes

- Depends on `validator::find_spec_files`.
- Dry-run returns the complete plan without staging or publishing writes.
- Summary rows end with `<!-- specsync:compact:v1 -->` and use `—` cells for wider tables.
- Rewriting preserves every untouched line and its exact terminator.
