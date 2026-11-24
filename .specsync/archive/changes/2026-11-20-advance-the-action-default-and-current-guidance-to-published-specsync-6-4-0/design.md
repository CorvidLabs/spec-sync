---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-0
artifact: design
---

# Design

- On exactly the lines #1069 moved for 6.3.0, the current release becomes 6.4.0 and the immutable tag's embedded default becomes 6.3.0: `action.yml`'s default and its description example, `PUBLISHED_ACTION_DEFAULT`, the Action docs' `version` row and published-binary text, README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md (install, CI pins, Trust `specsync-version`), the site index, the manual-download URL, and the github testing row.
- The two sentences the 6.4.0 preparation left saying "after publication" or "candidate" now read as published, as #1069 did.
- Unchanged: historical text (the 6.2.0 floor, the relayed-decision requirement, the comment-fallback note for `@v6.3.0` and later), the composite runtime scripts, the CI and Trust candidate mirrors, and every immutable tag.
- The github spec Purpose moves through one SpecSection delta. CHANGELOG `[Unreleased]` gains the entry, and `docs/releases/6.4.0.md` records publication.
