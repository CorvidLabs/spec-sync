---
change: import-from-dir-review-follow-ups
artifact: context
---

# Context






Round-2 review of #416 (PR #1022) found: parse_body_sections split on `## ` inside fences;
depends_on/db_tables/implements and the H1 lead-in were dropped; source `status` was copied
verbatim against the "imports stay draft" constraint; and refusing frontmatter-less docs
narrowed HI SETUP-5 — they import as warned drafts instead.
