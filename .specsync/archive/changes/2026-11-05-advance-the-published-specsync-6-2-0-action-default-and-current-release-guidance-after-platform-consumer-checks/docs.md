---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: docs
---

# Docs

README, `site/src/content/docs/integrations/github-action.md` (default row, preparation paragraph, manual-CI download) and the `action.yml` input description say the main wrapper defaults to published 6.2.0, that immutable `@v6.2.0` embeds default 6.1.2 so pass `version: '6.2.0'` with it, and that floating `@v6` advances only after consumer checks of this wrapper commit. The immutable `@v6.0.0` caveat stays. SECURITY.md, MIGRATION.md current pins, docs/ADOPTING.md, the site index, quickstart, ci-gate and polyglot drop candidate notes and name 6.2.0. cli.md and workflow.md say `change reconcile-history` is available in 6.2.0. CHANGELOG dates 6.2.0 and gets an Unreleased entry; docs/releases/6.2.0.md records publication and checks off completed gates.
