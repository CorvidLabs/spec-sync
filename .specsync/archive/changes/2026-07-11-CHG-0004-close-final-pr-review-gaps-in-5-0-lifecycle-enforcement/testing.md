---
change: CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-012 | Unit regressions for lifecycle coverage states, dirty-tree union, closing evidence, malformed/extra deltas, accept-time tombstones, semver bumps, and transitive dependencies; existing empty-diff regression |
| REQ-cmd-check-001 | Integration regression asserting the stable top-level JSON shape on an SDD failure |
| REQ-cmd-init-003 | Init unit/integration regressions for detected source policy scopes and committed policy/config meaningful paths |

Focused regressions run first, followed by CI-style parallel and serial unit tests, the full repository lane, executable examples, and the GitHub matrix.
