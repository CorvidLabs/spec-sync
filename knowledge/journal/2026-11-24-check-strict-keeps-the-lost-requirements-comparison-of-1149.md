---
id: 2026-11-24-check-strict-keeps-the-lost-requirements-comparison-of-1149
title: "check --strict keeps the lost-requirements comparison of #1149 when…"
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-24
time: 2026-11-24T14:47:48Z
kind: shipped
links:
  - 1149
  - .chough/archive/sessions/2026-11-24-1149-strict-check-policy-flip.md
  - .specsync/archive/changes/2026-11-24-keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off/
record: .chough/archive/sessions/2026-11-24-1149-strict-check-policy-flip.md
actor:
  - agent: claude-code
    terminal: t14v
---
# check --strict keeps the lost-requirements comparison of #1149 when…

check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)

Why: 0xLeif chose 'Fix as a successor change (Recommended)' when asked what to do with the Codex P1 on #1149 (Claude Code release chat, 2026-11-24 about 04:42 UTC). The option approved the scope shown with it (cmd_check and change; check.rs, one helper in change, the vanished-requirements tests, specs/cmd_check/, specs/change/; MODIFIED REQ-cmd-check-004) and archival once the full native Verify passes.

Verified: Full ten-step native Verify lane at the scoped commit 303f1d85: 3,336 unit tests passed with 2 ignored, all 812 integration tests passed, the release build, strict spec check and the…

Read more: the pull request 1149 · [the story](../../.chough/archive/sessions/2026-11-24-1149-strict-check-policy-flip.md) · [the archived change](../../.specsync/archive/changes/2026-11-24-keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off/)
