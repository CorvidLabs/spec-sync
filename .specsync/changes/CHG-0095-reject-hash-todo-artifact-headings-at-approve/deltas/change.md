## ADDED

### REQUIREMENT REQ-change-054

Change artifact completeness SHALL treat HTML TODO comments, bare TODO lines, and markdown headings whose title is only TODO (optionally with a trailing description after a colon) as incomplete placeholder content.

Acceptance Criteria

- `change approve` rejects when any selected artifact body is empty or only placeholder TODO content after YAML frontmatter.
- `change status` / next-action guidance list those incomplete artifact paths and do not recommend approve.
- Artifacts with real prose or completed checklist items remain complete even when a section heading is present.
- HTML TODO comments continue to mark an artifact incomplete.
