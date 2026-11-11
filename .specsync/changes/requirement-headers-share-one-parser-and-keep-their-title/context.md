---
change: requirement-headers-share-one-parser-and-keep-their-title
artifact: context
---

# Context

Round 2 of #1024 (PR #1039), from t384's independent review. Round 1 hand-
rolled an id-shape guard narrower than the real parsers: `REQ-cmd-change-012`
(multi-segment module) and `REQ-x-001: Title` (punctuation after the id) were
still refused. Leif's call via t370: use the same id rule as
`is_requirement_id`, and keep the title — #1024 asked for it to be kept.

The fix: `requirement_heading_id` (validator) becomes `pub(crate)` and the
materializer's matcher delegates to it for `### ` targets; `## ` section keys
keep exact matching. On write, the matched header line is written back
unchanged, so the title survives; convergence comes from the tolerant matcher,
not from normalization. REQ-change-150's wording is updated by the delta.
