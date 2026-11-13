---
change: a-delta-can-add-replace-or-remove-a-whole-section-of-requirements-md
artifact: plan
---

# Plan

## Approach

A third delta target that reuses SPEC SECTION's machinery on `requirements.md`, with a section
boundary that stops at requirement headings.

## Out of scope

Line-level edits inside a list (Leif chose whole sections) and `### REQUIREMENT` behaviour.

## Steps

1. Failing tests: parse, MODIFIED, the requirement boundary, REMOVED, ADDED with convergence and
   refusal, and the requirement-heading refusal.
2. The variant, its parse, every target mapping, the boundary, and the validation.
3. The site's delta reference names the third block kind, with an example.

## Risks

A `## <name>` heading that appears twice in `requirements.md` targets the first, as SPEC SECTION
does in the spec.

## Constraints consulted

REQ-change-152 (the shared requirement-heading reader) and the fenced-code rule (#1050).
