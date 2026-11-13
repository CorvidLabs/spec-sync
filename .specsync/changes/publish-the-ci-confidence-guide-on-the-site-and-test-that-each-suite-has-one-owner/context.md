---
change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
artifact: context
---

# Context

#493, a follow-up from PR #491's review, bounded to two omissions: publish the CI confidence guide
on the site, and add an executable check that the Trust lifecycle runs no duplicate suite. Leif
chose to build it on 2026-11-13 ("#493 CI guide on the site", coordinator chat 03:49Z, relayed by
claude-code-t370). REQ-github-006 already states the rule, so there is no new contract text.

The guide moves to `site/src/content/docs/ci-confidence.md` (Reference, order 7). Its links are
absolute, so they survive the move. `docs/ci-confidence.md` becomes a pointer, so AGENTS.md,
CONTRIBUTING.md and `.trust.toml` links still resolve. The hub keeps its own menu, so its entry is
the one CorvidLabs/site issue the coordinator asked for, shared with #1003.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
