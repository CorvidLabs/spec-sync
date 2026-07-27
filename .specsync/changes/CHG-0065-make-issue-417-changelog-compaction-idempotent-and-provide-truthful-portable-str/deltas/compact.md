## MODIFIED

### REQUIREMENT REQ-compact-001

The compact module SHALL compact only excess Change Log rows while preserving recent entries,
table structure, byte stability, and dry-run safety.

Acceptance Criteria

- Only the `## Change Log` slice is rewritten; header and separator rows remain intact.
- The last `keep` ordinary rows are retained and older ordinary rows become one well-formed summary.
- A generated summary is recognized only by its exact provenance marker, range, placeholders, and
  grammatical checked fixed-width count.
- New excess rows fold prior generated counts while retaining the original range start.
- Unmarked lookalikes remain ordinary history; multiple marked summaries fail closed.
- Odd escaped pipes and code-span pipes stay inside cells; even backslash runs expose delimiters.
- Only the first contiguous width-valid table is compacted.
- No-op repeats are byte-for-byte idempotent; changed runs preserve exact LF/CRLF terminators.
- Dry-run returns results without writes, and only results with removed rows are surfaced.
- `CompactResult.compacted_entries` counts retained ordinary rows and excludes the summary.
- Every input is inspected and same-directory replacement staged before publication; failures are
  structured and incomplete/partial work never claims success.

### SPEC SECTION Invariants

8. Re-running compaction with the same `keep` value is byte-for-byte idempotent.
9. Escaped/code-span pipes and exact LF/CRLF terminators are preserved.
10. Only provenance-marked summaries are folded; duplicate markers and overflow fail closed.
11. Preflight/staging completes before publication and failures remain explicit.
