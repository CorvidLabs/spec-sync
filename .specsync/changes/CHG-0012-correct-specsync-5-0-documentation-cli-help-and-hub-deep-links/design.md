---
change: CHG-0012-correct-specsync-5-0-documentation-cli-help-and-hub-deep-links
artifact: design
---

# Design

Keep the change documentation-only except for correcting Clap help strings.
Preserve the existing CLI grammar and lifecycle state machine. Standalone docs
routes should map their requested slug to the same slug under the CorvidLabs
documentation hub rather than collapsing every page to the hub root.

No data format, command argument, validation rule, or extraction behavior
changes.
