## ADDED

### REQUIREMENT REQ-output-002

Markdown check output SHALL accept planned-mapping notices and render a distinct Planned Mappings section.

Acceptance Criteria

- The canonical `print_check_markdown` signature includes the notice collection.
- Planned mappings are separate from errors and warnings.
- The notice section does not alter validation state or pass/fail decisions.
