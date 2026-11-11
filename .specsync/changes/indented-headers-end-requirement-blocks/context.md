---
change: indented-headers-end-requirement-blocks
artifact: context
---

# Context

t384's round-3 finding on #1039: round 2's matcher accepts headers indented up
to 3 spaces (the parser's ATX rule), but `markdown_block_range`'s end scan
used a column-0-only heading test, so a MODIFIED/REMOVED on an indented header
ran through the next indented requirement and deleted it. The end scan now
uses the same ATX indent rule. Also: the kept header line is preserved
byte-for-byte (only the line ending's `\r` comes off), and the record
corrects the title-decision attribution to coordinator direction matching
#1024 — not a Leif decision.
