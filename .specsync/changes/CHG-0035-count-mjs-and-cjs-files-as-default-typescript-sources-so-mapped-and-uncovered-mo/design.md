---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: design
---

# Design

Treat `.mjs` and `.cjs` as TypeScript-family inputs, matching the existing handling of `.js`, `.jsx`, `.mts`, and `.cts`. Add both suffixes to the two authoritative `Language` surfaces: `from_extension` for detection and `extensions` for default discovery. This preserves explicit `source_extensions` filtering, parser selection, and public language naming while closing the denominator gap.

Coverage behavior requires no special case: once the shared registry recognizes the files, existing validator discovery and LOC measurement include them naturally. Regression fixtures assert exact totals so a future classification mismatch cannot produce another vacuous 100 percent result.
