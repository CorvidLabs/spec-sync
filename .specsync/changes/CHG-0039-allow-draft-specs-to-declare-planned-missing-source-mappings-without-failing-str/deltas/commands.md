## ADDED

### REQUIREMENT REQ-commands-002

Check reporting SHALL expose planned mapping notices separately from warnings in text, JSON, Markdown, and GitHub formats.

Acceptance Criteria

- `run_validation` returns deterministic notice strings as a seventh tuple member.
- Text output identifies each planned path without printing a misleading all-files-exist check.
- Structured JSON includes a deterministic notices array.
- Markdown and GitHub reports include a planned mappings section.
- Notice-only results remain passing under strict enforcement.
