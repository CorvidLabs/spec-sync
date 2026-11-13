---
change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
artifact: context
---

# Context

#1003 was split from #831, which closed once plain `specsync change status <id> --json` met its
cost goal (#918, #979, #989). What #831 also asked for was still missing: a documented, supported
way for CI to check one archived change, with its trust model. Leif chose to build it on
2026-11-13 ("#1003 CI archive check docs", coordinator chat, about 03:28Z, relayed by
claude-code-t370). The coordinator allocated REQ-cmd-change-038.

The page is `site/src/content/docs/integrations/ci-archive-check.md`, linked from the Action
page. Each trust-model sentence was checked against the code: `read_cache.rs` turns the
persistent cache off for shallow, grafted, replace-ref and partial clones, and REQ-change-148
covers the cleared history-rewriting environment. The site's standalone pages redirect to the
CorvidLabs hub, which copies `site/src/content/docs` verbatim and keeps its own navigation
registry. Publishing the page there needs a registry entry in CorvidLabs/site (a follow-up).

Chough: Leif's rule (relayed by t370 on 2026-11-13) is "make sure u use chough 0.6.0 for all
spec-sync changes". spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges, so this
change is built now and goes through a Chough 0.6.0 session before review.
