---
id: bind-discovered-cargo-crate-sources-during-generation
state: archived
type: bug_fix
base_commit: 9003fad7ec4dea3938179d076248b48f29b6ed48
---

# Bind discovered Cargo crate sources during generation

## Intent

Bind discovered Cargo crate sources during generation

## Affected Canonical Specs

- `generator`
- `validator`
- `types`
- `cli`
- `output`
- `comment`

## Acceptance Criteria

- The issue #813 minimal Cargo library succeeds through init then generate, creates specs/demo/demo.spec.md bound to src/lib.rs and documents add. Text, JSON, and batch generation bind discovered Cargo sources; binary and workspace crates are scoped correctly; existing spec-owned files remain owned; missing configured modules still fail with skipped_no_files.

## No-spec Rationale

Not applicable
