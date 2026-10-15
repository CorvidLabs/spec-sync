## ADDED

### REQUIREMENT REQ-cmd-new-005

The new command SHALL bind exact configured root Cargo source files through the same safe selection semantics as other creation commands.

Acceptance Criteria
- A root lib.rs or main.rs is truthfully bound when unambiguous; explicit mappings and directory modules retain behavior; filters, confinement and ambiguity are preserved.
