---
change: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
artifact: docs
---

# Docs

- Add a dedicated canonical `exports` spec section for direct CommonJS assignments and object exports.
- Add a behavioral example showing supported static names and intentionally ignored dynamic forms.
- Record the implementation and test coverage in the exports companion files.
- No CLI syntax or user guide changes are required; this corrects extraction behavior for already discovered `.cjs` files.
