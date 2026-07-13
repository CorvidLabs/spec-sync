---
change: CHG-0022-preserve-canonical-change-log-table-schemas-when-accepting-semantic-deltas
artifact: context
---

# Context

Acceptance currently appends a fixed `Date | Change` row after applying a semantic delta. Repositories may
legitimately use other canonical Change Log schemas, including `Version | Date | Changes` and
`Date | Author | Change`. The fixed row creates the wrong number and order of cells and, in versioned tables,
omits the canonical version that acceptance just incremented. A real Canary WASM correction reproduced the
defect: accepting the correction recreated a malformed two-cell row beneath a three-column header.

The correction derives the appended row from the existing table header. It fills the current post-bump version,
the current date, a stable SpecSync author label when requested, and the accepted change description. Unknown
columns remain empty instead of shifting recognized values into the wrong column. Specs without a usable table
retain the established two-column fallback.
