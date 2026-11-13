---
kind: bug
id: 2026-11-06-change-status-on-an-archived-change-outlasts-the-chough-native
aliases:
  - BUG-change-status-on-an-archived-change-outlasts-the-chough-native-timeout-01f633e1de1d1cf3d306282f
title: change status on an archived change outlasts the Chough native timeout
status: open
classification: unclassified
scope: in-scope
impact: Chough cannot record accept/done for an archived spec-sync change; delivery is unaffected because native state is authoritative. The timeout was not raised.
criteria: []
requirements: []
cases: []
evidence: []
receipts: []
environment: local-macos-arm64
revision: 7eb92d10d3978cfb11efa00f2f56376466591f11
found_by: manual report
related: []
created: 2026-11-06T14:57:15Z
updated: 2026-11-06T14:57:15Z
---
# change status on an archived change outlasts the Chough native timeout

**What happened:** chough stage advance (test -> accept) is refused native-unknown after its native status read times out
Actual: specsync change status <id> --json took 4m56s (135.8 s user, 274.5 s system) with specsync 6.2.0 built from this branch; chough stopped it at 120 s and refused the advance

**Steps:** Trigger: chough stage advance after specsync change ship archived the change describe-the-cli-recursion-guard-... at 7eb92d10
1. specsync change ship <id> (archives the change)
2. chough stage advance
3. time specsync change status <id> --json

**Expected:** specsync change status <id> --json answers within CHOUGH_NATIVE_TIMEOUT_SECONDS (120 s), so the archived state can be read and the stage can advance

**Why it matters:** Chough cannot record accept/done for an archived spec-sync change; delivery is unaffected because native state is authoritative. The timeout was not raised.

**Fix:** Not fixed yet.

## Log

- 2026-11-06T14:57:15Z · Reported (manual report)
