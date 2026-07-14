---
change: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
artifact: plan
---

# Plan

1. Add a comment- and string-aware lexical CommonJS extractor to the TypeScript/JavaScript regex backend.
2. Merge CommonJS names into regex results with stable deduplication.
3. Reuse the same helper from the AST backend so successful ESM parsing does not suppress CommonJS symbols.
4. Add focused regressions for direct properties, object shorthand/named keys, mixed modules, deduplication, and ignored dynamic syntax.
5. Apply the `exports` semantic delta and update its companions.
6. Run focused and complete tests, strict spec validation, hosted CI, Trust, and Attest verification.
