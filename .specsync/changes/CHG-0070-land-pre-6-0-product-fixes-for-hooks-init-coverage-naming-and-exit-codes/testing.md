---
change: CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes
artifact: testing
---

# Testing

## Automated evidence

- `cargo test` (full unit + integration suite) — green on this tip.
- Requirement coverage:
  - REQ-hooks-001 — hooks managed-block / git hooks dir tests
  - REQ-commands-001 — command exit-code and module naming tests
  - REQ-validator-001 — draft-planned coverage tests
  - REQ-parser-001 — frontmatter validation tests
  - REQ-registry-001 — registry TOML parsing tests
  - REQ-config-001 — init/config honesty tests
  - REQ-ignore-001 — ignore rule tests
  - REQ-exports-001 — export extraction tests
  - REQ-scoring-001 — score gate tests
  - REQ-hash-cache-001 — hash cache tests
  - REQ-cli-001 — CLI dispatch tests
  - REQ-cmd-check-001 — check command tests
  - REQ-cmd-coverage-001 — coverage command tests
  - REQ-cmd-score-001 — score command tests
  - REQ-agents-001 — agents install tests
