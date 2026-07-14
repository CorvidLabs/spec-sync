## ADDED

### REQUIREMENT REQ-comment-002

GitHub check comments SHALL accept planned-mapping notices as an explicit renderer input and present them separately from errors and warnings.

Acceptance Criteria

- The canonical `render_check_comment` signature includes the notice collection.
- Planned mappings render in their own section.
- Notice-only results remain passing and do not inflate warning totals.
