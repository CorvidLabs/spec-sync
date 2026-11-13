---
kind: bug
id: 2026-11-07-late-close-follows-destination-symlinks-outside-the-project
aliases: [BUG-late-close-follows-destination-symlinks-outside-the-project-69c842b5326c22b0c1dbafd3]
title: Late close follows destination symlinks outside the project
status: verified
classification: product-defect
scope: in-scope
impact: P1 loss of the only active package from repository history
criteria: [SHIP-12]
requirements: [REQ-change-143]
cases: []
evidence:
  - OBS-product-test-8a9f5a47598253cb2208e85c
  - OBS-product-test-943d52f230af9865e7c1fe58
  - OBS-product-test-d753cf8983ee0af03f0458f9
  - OBS-product-test-a44413a32a903e5b908007e8
receipts: []
fixture: Disposable same-filesystem work/origin/outside fixtures in tests/integration/doctor.rs; direct writer matrix in src/change_tests/doctor_collision.rs
environment: local-macos-arm64
revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
found_by: "PR #994 review thread PRRT_kwDORqhp_860xF_E; comment 4499656268"
related: []
created: 2026-11-07T06:51:20Z
updated: 2026-11-07T06:58:59Z
---
# Late close follows destination symlinks outside the project

**What happened:** Active packages disappear from the project through linked archive or closed-late directories
Actual: Red tests reproduce successful outside moves and dangling destinations accepted by dry-run

**Steps:** Trigger: doctor --fix --close-late with a same-filesystem directory symlink in the destination path

**Expected:** Refuse in dry-run and at the write boundary, preserving source, ledger and outside bytes

**Why it matters:** P1 loss of the only active package from repository history

**Fix:** Recorded before 0.6.0; the Log keeps its history.

## Log

- 2026-11-07T06:51:20Z · Reported (PR #994 review thread PRRT_kwDORqhp_860xF_E; comment 4499656268)
- 2026-11-07T06:51:41Z · status: open -> verified. All five new destination-symlink regressions failed first, then passed with the component checks; focused controls pass. Full verify and hosted gates remain pending.; evidence: OBS-product-test-8a9f5a47598253cb2208e85c, OBS-product-test-943d52f230af9865e7c1fe58
- 2026-11-07T06:58:59Z · Recovery refinement failed first, then all five regressions passed. Final-component retry removes only the link and leaves the dated destination absent; parent recovery remains tested.; evidence: OBS-product-test-d753cf8983ee0af03f0458f9, OBS-product-test-a44413a32a903e5b908007e8
