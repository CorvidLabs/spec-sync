---
id: 2026-11-20-specsync-6-4-0-release-preparation-on-main
title: SpecSync 6.4.0 release preparation on main
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T02:19:24Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-release-6-4-0-prep.md
  - .specsync/archive/changes/2026-11-20-prepare-specsync-6-4-0-package-metadata-and-release-handoff/
record: .chough/archive/sessions/2026-11-20-release-6-4-0-prep.md
actor:
  - agent: claude-code
    terminal: t59
---
# SpecSync 6.4.0 release preparation on main

SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish

Why: Authorization: ss-lead (hc t58) assigned the 6.4.0 release-prep PR on 2026-11-20, from its release plan that Leif's answers set (relayed by hc t4c, 2026-11-19 about 15:49Z and 15:53Z). Leif's definition approval, review and yes to merge are his own; no tag, release, publish or workflow dispatch here.

Verified: One rerun of the fixture after its version restore stopped using BSD-only sed -i '' (it now writes through a temporary file, so GNU sed runs it too), with this branch's own release build (specsync 6.4.0, built at 68b76f27) against Homebrew's released 6.3.0: the same 26 commands and exit codes as the earlier run with main 10d9bb3f's build, which…

Read more: [the story](../../.chough/archive/sessions/2026-11-20-release-6-4-0-prep.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-prepare-specsync-6-4-0-package-metadata-and-release-handoff/)
