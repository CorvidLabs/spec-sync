## MODIFIED

### REQUIREMENT REQ-cmd-scaffold-006

`scaffold` SHALL exclude files already claimed by an existing spec's `files:` list from the scaffolded spec's `files:` list.

Acceptance Criteria
- `scaffold foo` where `src/foo.rs` is already listed by another spec never lists `src/foo.rs`.
- A module with no unowned candidate files takes the existing no-files path: the spec is written with `files: []` and the empty-files warning.
