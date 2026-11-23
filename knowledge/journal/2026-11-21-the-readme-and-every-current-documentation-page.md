---
id: 2026-11-21-the-readme-and-every-current-documentation-page
title: The README and every current documentation page
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-21
time: 2026-11-21T15:29:37Z
kind: shipped
links:
  - 1141
  - .chough/archive/sessions/2026-11-21-docs-6-4-accuracy.md
  - .specsync/archive/changes/2026-11-21-correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples/
record: .chough/archive/sessions/2026-11-21-docs-6-4-accuracy.md
actor:
  - agent: claude-code
    terminal: t81
---
# The README and every current documentation page

The README and every current documentation page, Mermaid diagrams included, say what SpecSync 6.4.0 and 6.4.1 do: the 21 items in defs/docs-6.4-accuracy-definition.md, documentation only, no version or pin wording (the 6.4.1 post-publication PR owns it); scope confirmed by t4c under Leif's standing OK for scope approvals

Why: No intent criterion: a documentation-only change that alters no behavior; the confirmed definition proposes none.

Verified: Documentation only, the 21 items of defs/docs-6.4-accuracy-definition.md (audit of main 3f0985e8 with its 6.4.1 build). Command check: every specsync invocation the docs show is valid against --help (80 command paths; only prose or placeholder hits left; the four check --commit hits gone).

Read more: the pull request 1141 · [the story](../../.chough/archive/sessions/2026-11-21-docs-6-4-accuracy.md) · [the archived change](../../.specsync/archive/changes/2026-11-21-correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples/)
