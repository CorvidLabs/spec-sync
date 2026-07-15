---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: context
---

# Context

SpecSync's schema scanner already recognizes `.mjs` and `.cjs` as JavaScript module files, but the shared `Language` classifier omits both suffixes. Default source discovery is derived from `Language::from_extension` and `Language::extensions`, so coverage silently excludes these files even when a canonical spec maps them.

The fix aligns the shared language registry with the existing schema behavior and proves both sides of the gate: mapped module files increase measured totals, and uncovered module files make strict 100 percent coverage fail.
