---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: design
---

# Design

Git calls the driver with the ancestor, the current file, and the other file. The result is written back to the current file. Exit 0 means clean. Exit 1 leaves conflict markers in place.

For a module spec, frontmatter lists `files`, `db_tables`, and `depends_on` are unioned and sorted. Numeric `version` becomes ancestor plus both deltas, never below either side. The change log unions rows and sorts them by the first cell, then the row text. Other prose that both sides changed differently stays a conflict.

For `requirements.md`, blocks are keyed by `### REQ-...`. An ID added on only one side is kept. The same new ID with two bodies is kept twice. An existing block edited on both sides stays a conflict.

`check` and `change audit` scan living `requirements.md` files and name any ID that appears more than once outside a fence.
