---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: docs
---

# Docs

- `site/src/content/docs/workflow.md`, "Relayed decisions": a change that holds a relayed decision is recorded at workflow version 3, so SpecSync 6.2.0 and older refuse it rather than drop the relay. Once a repository records one, every machine that runs change commands there needs 6.3.0 or later; `specsync check`, the GitHub Action and Trust are unaffected.
- `README.md`: the same sentence beside its relayed-decision paragraph.
- `CHANGELOG.md` `[Unreleased]`: the entry, with that sentence. The 6.3.0 release notes carry it too (the release-prep PR).
