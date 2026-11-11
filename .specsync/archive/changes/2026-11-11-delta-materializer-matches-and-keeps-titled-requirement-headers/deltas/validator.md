---
module: validator
---

## ADDED

### REQUIREMENT REQ-validator-047

Requirement-heading identity SHALL be decided by one shared parser: the
validator's `requirement_heading_id`, reused by the delta materializer, so a
header form the validator accepts is a form the materializer matches.

#### Acceptance Criteria

- The materializer's block matcher returns the same answer as
  `requirement_heading_id(line) == Some(key)` for every `### ` target.
