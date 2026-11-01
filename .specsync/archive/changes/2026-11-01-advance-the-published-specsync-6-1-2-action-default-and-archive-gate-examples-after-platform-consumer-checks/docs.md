---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: docs
---

# Docs

README, `site/src/content/docs/integrations/github-action.md` (default row and archive-gate example) and the `action.yml` input description say the main wrapper defaults to published 6.1.2, that immutable `@v6.1.2` embeds default 6.1.1 so pass `version: '6.1.2'` with it, and that floating `@v6` advances only after consumer checks of this wrapper commit. The immutable `@v6.0.0` caveat stays. CHANGELOG gets an Unreleased entry; docs/releases/6.1.2.md checks off completed gates.
