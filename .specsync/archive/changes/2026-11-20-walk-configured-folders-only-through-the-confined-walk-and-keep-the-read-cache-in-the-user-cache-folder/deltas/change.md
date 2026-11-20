## ADDED

### REQUIREMENT REQ-change-164

`change adopt`'s requirement ID proposals SHALL find `requirements.md` files under `specs_dir` only through the confined, budgeted walk (REQ-validator-053), and the read cache SHALL be read and written only in the folder REQ-util-009 names.

Acceptance Criteria
- A committed `specs` link that leaves the project yields no proposals, and the read cache of a Git work tree is outside the clone.
