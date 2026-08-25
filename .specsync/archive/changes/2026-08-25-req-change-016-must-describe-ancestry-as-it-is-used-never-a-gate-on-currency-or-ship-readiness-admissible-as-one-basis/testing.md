---
change: req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis
artifact: testing
---

# Testing

This change edits one canonical requirement and no source, so the evidence is materialization
equality plus the unchanged suite.

- `specsync change check <id>` materializes `deltas/change.md` into `specs/change/requirements.md`
  and runs the configured commands, including `cargo test`. A green run proves no behaviour moved.
- Byte-equality assertion: the body under `### REQ-change-016` in the materialized
  `specs/change/requirements.md` is compared line-for-line (whole lines, not substrings) with the
  body under `### REQUIREMENT REQ-change-016` in `deltas/change.md`. This is asserted after
  materialization, not before.
- Neighbour check: `git diff specs/change/requirements.md` touches exactly the one bullet — two
  lines removed, six added — and no other requirement in the file.
- Delta-integrity check: `deltas/change.md` contains exactly one `## MODIFIED` block and exactly
  one `### REQUIREMENT` item, so nothing can be dropped by a regeneration that stops early. The
  file was produced by a script that compares whole lines and asserts the replaced bullet is
  present exactly once before writing.
- Not tested here: that the code obeys the new MUST NOT clause. The two conjunct sites at
  `src/change.rs:13874` and `:13879` violate it today; that is #706's discriminating test, not
  this change's.
