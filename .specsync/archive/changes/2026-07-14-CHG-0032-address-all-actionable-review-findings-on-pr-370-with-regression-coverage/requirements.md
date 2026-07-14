---
change: CHG-0032-address-all-actionable-review-findings-on-pr-370-with-regression-coverage
artifact: requirements
---

# Requirements

- Historical accepted evidence remains byte-stable when later sequence owners acknowledge additional collisions.
- Verification preflight rejects direct SpecSync execution, implicit default checks, and Cargo commands that explicitly select the SpecSync package.
- Both supported local registry paths are protected lifecycle inputs.
- An affected module covers exactly its resolved canonical spec and requirements companion unless the change explicitly lists broader paths.
- Cargo package detection accepts ordinary TOML whitespace, literal strings, and trailing comments.
