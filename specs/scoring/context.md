---
spec: scoring.spec.md
---

## Key Decisions

- **5 equal components**: Each dimension (frontmatter, sections, API coverage, content depth, freshness) is worth exactly 20 points for a total of 100. Equal weighting keeps the rubric simple and predictable.
- **Letter grades**: A (90+), B (80-89), C (70-79), D (60-69), F (<60). These map to intuitive quality tiers.
- **TODO counting ignores code blocks**: Placeholders like `TODO` and `<!-- ... -->` are only counted outside fenced code blocks, preventing code examples from penalizing the score.
- **No-exports = full API score**: Modules with no extractable exports (e.g., config-only, types-only) receive full API coverage points rather than being penalized for having nothing to document.
- **Stale reference penalties**: Missing source files cost 5 points each (max 15), missing dependency specs cost 3 points each. This encourages keeping specs in sync with code changes.
- **Actionable suggestions**: Each score includes specific improvement suggestions (e.g., "Add version field to frontmatter") so users know exactly what to fix.

## Files to Read First

- `src/scoring.rs` — Single-file module with scoring logic, grade calculation, and project-level aggregation.

## Current Status

Fully implemented. Individual spec scoring and project-level aggregation both work. The MCP server exposes scoring through the `specsync_score` tool.

## Notes

- Content depth checks for "meaningful content" — sections with only headings, HTML comments, or table separator rows (`|---|`) don't count as having content.
- Project scores include a grade distribution (count of A/B/C/D/F specs) for quick triage.
