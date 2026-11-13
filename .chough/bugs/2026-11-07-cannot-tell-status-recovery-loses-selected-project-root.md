---
kind: bug
id: 2026-11-07-cannot-tell-status-recovery-loses-selected-project-root
aliases: [BUG-cannot-tell-status-recovery-loses-selected-project-root-11cb6d86442c49424dda6f96]
title: Cannot-tell status recovery loses selected project root
status: verified
classification: product-defect
scope: in-scope
impact: A user recovering from unreadable upstream archives cannot finish the advertised recovery from the original directory
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
cases: []
evidence: [OBS-product-test-2d6da6e25125f8c0fc3cf3a8, OBS-product-test-efd35a9f612ba24bb9c3ae66]
receipts: []
environment: local-macos-arm64
revision: 23fffcedee001dac6ede47e89ffe912b819b4552
found_by: "PR #967 round-2 P1 executable recovery regression"
related: []
created: 2026-11-07T06:51:12Z
updated: 2026-11-07T06:51:33Z
---
# Cannot-tell status recovery loses selected project root

**What happened:** Printed status rerun from the project parent cannot find the change
Actual: At 23fffcedee001dac6ede47e89ffe912b819b4552 the final printed command exited 1: change not found

**Steps:** Trigger: Run all printed recovery commands after specsync --root selects a nested project

**Expected:** Status retries the same selected project and names the upstream archive

**Why it matters:** A user recovering from unreadable upstream archives cannot finish the advertised recovery from the original directory

**Fix:** Recorded before 0.6.0; the Log keeps its history.

## Log

- 2026-11-07T06:51:12Z · Reported (PR #967 round-2 P1 executable recovery regression); evidence: OBS-product-test-2d6da6e25125f8c0fc3cf3a8
- 2026-11-07T06:51:33Z · status: open -> verified. The executable regression now runs the printed status command against the selected root from the repository parent; all 9 focused unit/command and 4 integration tests passed in t1b2. No native approval/review or second archive was written.; evidence: OBS-product-test-efd35a9f612ba24bb9c3ae66
