---
change: tell-agents-when-it-is-safe-to-clear-context
artifact: docs
---

# Docs

`site/src/content/docs/workflow.md` gains a "Clearing context" section: what `safe`,
`conditional`, and `not yet` mean, why approval and `check --commit` are the clean boundaries,
and why an uncommitted `review.json` does not count against you. `site/src/content/docs/cli.md`
shows the `Handoff:` line in the status output. `CHANGELOG.md` records the new line and the
`summary.handoff` JSON field. Generated agent skills carry the one-sentence rule.
