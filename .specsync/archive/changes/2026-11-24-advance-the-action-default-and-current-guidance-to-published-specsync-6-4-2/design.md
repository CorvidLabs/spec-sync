---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-2
artifact: design
---

# Design

- On exactly the lines #1142 moved for 6.4.1, the current release becomes 6.4.2 and the immutable tag's embedded default becomes 6.4.1: `action.yml`'s default and its description, `PUBLISHED_ACTION_DEFAULT`, the Action docs' `version` row and published-binary text, README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md (install, CI pins, Trust `specsync-version`), the site index, the manual-download URL, and the github testing row.
- The three sentences the 6.4.2 preparation left reading "after publication" (one in README, two in the Action docs) read as published, in the words #1142 used.
- Unchanged: historical text (the 6.2.0 floor, the relayed-decision requirement, MIGRATION.md's notes on earlier releases, the comment-fallback note for `@v6.3.0` and later), the composite runtime scripts, the CI and Trust candidate mirrors, and every immutable tag.
- The github spec Purpose moves through one SpecSection delta. CHANGELOG `[Unreleased]` gains the entry, and `docs/releases/6.4.2.md` records publication.
