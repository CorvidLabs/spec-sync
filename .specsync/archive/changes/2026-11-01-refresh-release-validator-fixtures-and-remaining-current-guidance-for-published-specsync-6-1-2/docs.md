---
change: refresh-release-validator-fixtures-and-remaining-current-guidance-for-published-specsync-6-1-2
artifact: docs
---

# Docs

- site/src/content/docs/quickstart.md:254: replace the candidate note with plain 6.1.2 guidance.
- site/src/content/examples/polyglot.mdx:11 and ci-gate.mdx:11: drop the "only after publication; published release remains 6.1.1" notes.
- SECURITY.md:52: pin `v6.1.2` with `version: '6.1.2'`; the main wrapper defaults to published `6.1.2`; drop the preparation wording.
- MIGRATION.md:22, 23, 60, 75: published 6.1.2 binary, `--tag v6.1.2`, `specsync 6.1.2`, Homebrew serves 6.1.2 once homebrew-tap#40 merges (worded as the tap formula rather than a claim it already serves), `@v6.1.2` with `version: '6.1.2'`.
- site/src/content/docs/integrations/github-action.md:240: `releases/download/v6.1.2/`.
