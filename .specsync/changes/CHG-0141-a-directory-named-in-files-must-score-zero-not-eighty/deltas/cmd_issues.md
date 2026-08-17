## ADDED

### REQUIREMENT REQ-cmd-issues-003

Issue discovery SHALL classify a directory encountered through confined metadata the same way the path-based predicate does.

Acceptance Criteria
- A directory maps to the directory source-snapshot variant rather than to an unreadable or empty one.
- The classification agrees with `check` and with confined MCP validation, so one path cannot be a directory to one caller and an unreadable file to another.
