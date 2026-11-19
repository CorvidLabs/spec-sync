---
change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
artifact: context
---

# Context

#574: `check` time grew with the square of the number of undocumented exports (the issue measured
160 s for 400k; on 2026-11-11 a debug build doubled the count and took 4.2 times as long). The warning
loop in `src/validator/consumed_by_re.rs` named each undocumented export's file with
`exports_by_file.iter().find(...)`, a scan of every export per warning. Now the first file for each
symbol goes into a map once. The output is identical: the same warnings name the same first file.
Leif approved it on 2026-11-13 ("#574 fast check", coordinator chat 14:03Z, relayed by
claude-code-t370).

The only observable difference is time, so the red test compares 8,000 and 32,000 undocumented
exports on the same machine. On main the ratio was 9.0 (547 ms vs 4.91 s); the test allows under 7.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
