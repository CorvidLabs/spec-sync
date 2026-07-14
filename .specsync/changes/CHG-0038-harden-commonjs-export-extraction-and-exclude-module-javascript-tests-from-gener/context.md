---
change: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
artifact: context
---

# Context

Late review of CHG-0036 found static-analysis edge cases around chained assignments,
function-local `exports` aliases, regular-expression literals, and type-only class
filtering. The same review found that `new` and `scaffold` apply extension
discovery without the shared test-file exclusion used by coverage. These are
false-negative and false-positive risks in newly governed `.cjs` and `.mjs`
projects, so they must be corrected before merging the discovery rollout.
