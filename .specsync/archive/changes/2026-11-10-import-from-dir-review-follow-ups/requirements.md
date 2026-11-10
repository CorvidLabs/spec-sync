---
change: import-from-dir-review-follow-ups
artifact: requirements
---

# Requirements


- A `## ` line inside a fenced code block never splits a section.
- All frontmatter list fields survive import; nothing is dropped silently.
- Imported specs are always `status: draft`; a source's non-draft status is recorded in the Change Log row.
- A parseable markdown doc without frontmatter imports as a draft with the empty-files warning; only unreadable/unparseable files are errors.
