## ADDED

### REQUIREMENT REQ-cmd-change-011

Commands that enumerate active changes SHALL distinguish an empty project from a project whose workspaces could not be read, and SHALL never present a partial roster as a complete one.

Acceptance Criteria
- `list` and `status` print every readable change, then name each unreadable workspace with the reason including the offending path, and exit non-zero.
- The empty-project line is printed only when enumeration succeeded and found nothing, so a project whose only workspace is unreadable never reports itself as empty.
- `ship-status` reports the same roster and the same non-zero exit for the same tree, and its JSON carries the unreadable entries alongside the readable ones.
- JSON output is a single parseable document in both cases: the historical bare array while every workspace is readable, and an object carrying `changes` and `unreadable` once any workspace is not.
- `ship` and lifecycle commit resolution refuse to infer a target change while any workspace is unreadable, rather than selecting from the readable remainder.
- Sibling-active-change reporting counts unreadable workspaces as active, so an unreadable workspace is never reported as nothing else being in flight.
- A project with no active changes retains its existing empty-project output and zero exit status.
