## ADDED

### REQUIREMENT REQ-validator-016

No child of a JVM source root SHALL be treated as a module.

Acceptance Criteria
- Children of `src/main/kotlin` and its siblings are not modules.
- A package hierarchy therefore contributes no module names, removing the choice of segment that produced both the original defect and its first attempted fix.
- A monorepo laid out as `packages/<name>/src/main/kotlin` does not collapse into a single module.
