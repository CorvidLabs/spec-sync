---
kind: bug
id: 2026-11-20-configured-folder-walks-follow-a-link-out-and-the-read-cache
aliases: []
title: Configured folder walks follow a link out and the read cache lives in git
status: open
classification: unclassified
scope: in-scope
impact: Walks over files outside the project, unbounded for a link to /, with names or counts shown; forged archived-history passes in an archive that ships .git. Private, with GHSA-7q5v-8jcc-rx54
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-009, REQ-util-010, REQ-change-164]
cases: []
evidence: []
receipts: []
environment: local-macos-arm64 (APFS), rustc 1.89.0
revision: 38cc5e879a4c890a838368a50138a9135e6893f7
found_by: claude-code (hc t5r) (manual report)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-20T04:56:08Z
updated: 2026-11-20T04:56:08Z
---
# Configured folder walks follow a link out and the read cache lives in git

**What happened:** change adopt's requirement proposals, the duplicate requirement ID check, init-registry, the MCP issues walk, the wizard, the schema snapshot and check's schema inputs, scaffold's case check, a missing file's suggestion and source auto-detection walk or list a committed specs, source or schema link that leaves the project; the change read cache keeps passing verdicts in the Git common directory

Actual: On 4516f355 with change 8's tests (38cc5e87): all 6 new unit tests and the scaffold CLI test fail

**Steps:** Trigger: Run change adopt, scaffold, init-registry, check or the MCP issues tool in a clone that commits specs, lib or db as a link out, or in an archive that ships .git

**Expected:** A configured folder that leads out of the project is refused, not walked; no verdict is read from a Git directory

**Why it matters:** Walks over files outside the project, unbounded for a link to /, with names or counts shown; forged archived-history passes in an archive that ships .git. Private, with GHSA-7q5v-8jcc-rx54

**Fix:** Not fixed yet.

## Log

- 2026-11-20T04:56:08Z · Reported (manual report) · by claude-code (hc t5r)
- 2026-11-20T04:56:08Z · Added despite similar reports: Change 7's bug was state an archive ships in its git directory for the cache, journal and remote cache; this is the configured-folder walks and the change read cache, a separate store · by claude-code (hc t5r)
