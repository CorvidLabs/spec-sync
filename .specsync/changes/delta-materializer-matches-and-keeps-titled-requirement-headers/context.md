---
change: delta-materializer-matches-and-keeps-titled-requirement-headers
artifact: context
---

# Context

Issue #1024, re-cut onto a fresh id per t370's numbering allocation (#1026
keeps REQ-change-150; this fix takes REQ-change-152). Carries the three rounds
of PR #1039's work: the materializer matches requirement blocks through the
shared `requirement_heading_id` parser (t384's round-2 review), the block-end
scan shares the parser's ATX indent rule (t384's round-3 finding: an indented
header's block ran through the next indented requirement), and a matched
header line is written back unchanged — the title is kept (coordinator
direction, matching what #1024 asked).

Supersedes #1039, whose branch had the id collision with #1026 baked into
archived records. READY track: the requirement text is Leif's at merge.
