---
change: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
artifact: requirements
---

# Requirements

## REQ-cmd-view-002

Rendering a spec view SHALL NOT report success when it produced nothing, and SHALL name what
it could not find.

Acceptance Criteria
- A requested module that matches no spec is reported by name and exits non-zero.
- The report names a close match when one exists, and otherwise lists the modules that do exist.
- A spec that fails to render causes a non-zero exit rather than being reported and ignored.
- A requested module that exists is still rendered and exits zero.
- Running with no module filter is unchanged.
