---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: requirements
---

# Requirements

`REQ-parser-003` is added in `deltas/parser.md`: every frontmatter reader in `parser` recognizes a
delimiter LINE by one rule — exactly three dashes followed by nothing but whitespace — applied to
both ends of the block, in either line encoding, with the two ends free to disagree.

No new requirement for `change`. That module's behaviour changes only through the reader it already
delegates to, and the contract it owns — that artifact completeness reads through
`parser::strip_frontmatter` and what that now guarantees — lives in invariant 35, which
`deltas/change.md` rewrites.

## Public contract change

`strip_frontmatter`, `parse_frontmatter` and `parse_checked_issue_references` accept a delimiter
line padded with trailing whitespace where they previously did not. The change is strictly
widening for documents that were being misread, and strictly preserving for every shape that was
already handled:

- A document with well-formed `---` delimiters gets the same answer, byte for byte.
- A document with a padded OPENER moves from "no frontmatter" to "frontmatter" in all three
  readers at once, so no reader disagrees with another about what the document is.
- A document with a padded CLOSER stops losing the body prose above its first horizontal rule.
- A document whose first line is not exactly three dashes gets the same answer it got before.

`parse_checked_issue_references` also gains documents whose two delimiters carry different line
endings, which it previously reported as having no frontmatter at all.
