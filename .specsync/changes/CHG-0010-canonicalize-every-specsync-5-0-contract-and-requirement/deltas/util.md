## ADDED

### REQUIREMENT REQ-util-001

Shared utility helpers SHALL provide deterministic edit distance and resource-bounded regular-expression compilation without side effects.

Acceptance Criteria
- `levenshtein` returns character-based edit distance for UTF-8 strings, including empty-string inputs.
- `safe_regex` returns a compiled regex for valid patterns within configured size limits.
- `safe_regex` returns `None` for invalid syntax or patterns that exceed regex/DFA size limits.
- Utility helpers remain dependency-light and do not read project configuration or filesystem state.
