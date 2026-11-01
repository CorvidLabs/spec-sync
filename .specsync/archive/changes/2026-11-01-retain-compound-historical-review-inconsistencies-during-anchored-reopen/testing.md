---
change: retain-compound-historical-review-inconsistencies-during-anchored-reopen
artifact: testing
---

# Testing

## Acceptance Criteria

REQ-change-117: the compound discriminator failed before the fix; all four focused recovery methods pass afterward, including the pure-hash and finalization-digest refusal controls. The qualified source passed the full pinned-Rust lane: 2691 unit tests, 478 integration tests, 66 release-contract tests, 35 CI-gate tests, strict 62-module checks and 100 percent coverage. Branch source equality and exact-head hosted checks are separately verified.

All 146 Rust/build input files match the fully qualified source. SHA-256 of sorted path-to-SHA256 JSON: `3ecd8f53060037d3b713531a4d1d61eedc06899a4c9488f2160a1a8041438cdd`. Current-main release/CI workflow contract tests passed separately after the workflow-only base update.
