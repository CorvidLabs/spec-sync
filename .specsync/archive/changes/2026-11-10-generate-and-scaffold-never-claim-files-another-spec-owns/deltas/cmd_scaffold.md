## ADDED

### REQUIREMENT REQ-cmd-scaffold-006

`scaffold` SHALL exclude files already claimed by an existing spec's `files:` list from the scaffolded spec's `files:` list, and SHALL decline to write a spec when every candidate file is already owned.

Acceptance Criteria
- `scaffold foo` where `src/foo.rs` is already listed by another spec writes no spec claiming `src/foo.rs`.
- A module with no unowned candidate files produces no spec.
