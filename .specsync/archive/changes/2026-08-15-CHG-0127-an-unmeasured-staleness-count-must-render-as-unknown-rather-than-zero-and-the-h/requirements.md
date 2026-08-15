---
change: CHG-0127-an-unmeasured-staleness-count-must-render-as-unknown-rather-than-zero-and-the-h
artifact: requirements
---

# Requirements

`REQ-cmd-report-00N` — an unmeasured staleness count SHALL render as unknown,
never as zero, in every format.

`REQ-config-00N` — the configuration scanner SHALL report an unterminated header
as a load failure, using the wording shared with the unreadable-file shape.

Out of scope: parsing TOML constructs the scanner does not implement.
