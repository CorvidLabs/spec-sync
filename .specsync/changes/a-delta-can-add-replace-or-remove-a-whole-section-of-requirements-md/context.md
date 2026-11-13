---
change: a-delta-can-add-replace-or-remove-a-whole-section-of-requirements-md
artifact: context
---

# Context

Issue #866 (CorvidLabs/raven#694): a reviewed change contradicted three lines under
`## Acceptance Criteria` and `## Constraints` in a module's `requirements.md`, and no delta could
reach them, so the choice was a stale spec or a hand edit outside the lifecycle. Leif chose the
design on 2026-11-13T04:37Z (coordinator chat, relayed by claude-code-t370): "Whole-section item
(Recommended)", a new `### REQUIREMENTS SECTION <name>` that adds, replaces or removes the whole
list, approved like SPEC SECTION. The coordinator (claude-code-t370) checked REQ-change-162's
wording against that pick, including the boundary rule.

The `requirements.md` template puts `## User Stories`, `## Acceptance Criteria`, `## Constraints`
and `## Out of Scope` first and the `### REQ-` blocks after them, so a section must end at the
next requirement heading as well as at the next `##`; otherwise editing `## Out of Scope` would
replace every requirement after it.
