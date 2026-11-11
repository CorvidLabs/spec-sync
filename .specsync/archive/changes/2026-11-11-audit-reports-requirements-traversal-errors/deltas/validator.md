---
module: validator
---

## ADDED

### REQUIREMENT REQ-validator-046

The duplicate-requirement-ID audit SHALL report a specs-tree traversal error as
an audit error naming the path; it SHALL NOT silently skip a directory it
cannot read.

#### Acceptance Criteria

- An unreadable subdirectory under the specs tree produces an audit error
  naming the path.
- A readable tree produces no traversal errors.
