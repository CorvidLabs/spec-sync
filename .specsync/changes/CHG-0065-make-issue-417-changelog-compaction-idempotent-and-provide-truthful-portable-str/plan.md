---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: plan
---

# Plan

1. Capture the issue #417 regressions for repeated compaction, summary ownership, escaped pipes,
   table width, trailing newlines, retained counts, pluralization, dry-run behavior, and structured
   output.
2. Provenance-mark generated summaries; reject ambiguity/overflow/malformed tables; preserve exact
   LF/CRLF bytes and parse pipe escapes/code spans correctly.
3. Forward the resolved output format through the root dispatcher and implement truthful JSON and
   Markdown/GitHub renderers for compact and archive-tasks.
4. Normalize actual Windows separators without aliasing Unix filenames, and safely render hostile
   Markdown/GitHub paths.
5. Preflight and atomically stage compact/archive replacements; expose complete/partial failures and
   nonzero incomplete outcomes.
6. Synchronize all five canonical specs and companions.
7. Run targeted tests, formatting, lint, the full repository lane, strict 100% spec coverage,
   score gates, and trust verification.
8. Replay compact/archive flows in `CorvidLabs/spec-sync-sandbox`, obtain independent implementation
   and adversarial reviews, resolve all high/medium findings, and require green GitHub CI.
9. Present exact verification evidence for closing approval; accept and archive only after merge.
