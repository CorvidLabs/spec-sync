---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: plan
---

# Plan

1. Add `mjs` and `cjs` to the shared TypeScript-family extension classification and registry.
2. Add focused unit coverage for direct classification and registry parity.
3. Add integration fixtures proving mapped `.mjs` and `.cjs` files affect exact file and LOC totals and uncovered files of either suffix fail strict 100 percent coverage.
4. Apply canonical `types` and `validator` semantic deltas and document the correction in the changelog.
5. Run focused tests, strict 100 percent SpecSync validation, the complete Fledge verification lane, and hosted matrices.
