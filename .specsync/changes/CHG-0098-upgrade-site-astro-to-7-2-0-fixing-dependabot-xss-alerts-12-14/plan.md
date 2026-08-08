---
change: CHG-0098-upgrade-site-astro-to-7-2-0-fixing-dependabot-xss-alerts-12-14
artifact: plan
---

# Plan

1. Upgrade `site` deps to Astro 7.2.0 + MDX 7 + markdown-remark/satteri.
2. Point `markdown.processor` at `unified()` so `rewriteMdLinks` keeps working.
3. Run `bun test` and `bun run build` in `site/`.
4. Ship via SDD lifecycle (CHG-0098) and merge; Dependabot should auto-close #12–#14 after main re-scan.
