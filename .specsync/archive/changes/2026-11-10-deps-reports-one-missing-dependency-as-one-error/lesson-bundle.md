# Lesson bundle: deps-reports-one-missing-dependency-as-one-error

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Deps reports one missing dependency as one error
- **Kind**: BugFix
- **Specs**: deps
- **Paths**: src/deps.rs, tests/integration/regression_w1.rs
- **Acceptance**: a spec whose depends_on names a module with no spec yields exactly one deps error for it, the 'Dependency spec not found' verdict check gives, for bare and path entries; missing_deps still names the edge; the regression_w1 pin now expects one occurrence

## Evidence

- Verification commit: `a1493ced4e1e3124ac0501ac4a963a93b91b52be`
- Base commit: `5d20c7fb871e450e133e9b6c93422a0cfb978c61`
- Verified by: `specsync check --spec deps`

## From the change's context.md

# Context

Issue #606. For `depends_on: [nonexistent-module]`, `deps` printed two errors. Graph
building (`build_dep_graph_checked`) recorded the shared `validate_local_dependency`
verdict ("Dependency spec not found"), kept the edge so `missing_deps` could name it, and
then `validate_deps` added "depends on '...' but no spec exists" for the same edge.
`tests/integration/regression_w1.rs` pinned the count at 2 on purpose until this fix.

Graph building now also returns the edges whose entry it already reported, and
`validate_deps` skips its second message for those. The private builder returns a small
struct; the public `DepNode` and `build_dep_graph` are unchanged.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match an
existing requirement (the deps criterion "an error for each declared dependency") run the
full lifecycle with 0xLeif as approve and review actor.

## From the change's testing.md

# Testing

Unit tests in `src/deps.rs` and the `regression_w1` integration test run against temp
projects. Both new assertions fail on the previous behavior.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| deps acceptance criterion (one error per missing dependency) | unit `test_one_missing_dependency_is_one_error`, `test_validate_missing_dep`; integration `deps_duplicate_depends_on_entries_deduped`, `deps_scalar_depends_on_is_not_silently_dropped` |

## Where these lessons go

- `specs/deps/deps.3md`
