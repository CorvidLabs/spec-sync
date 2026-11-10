---
change: deps-reports-one-missing-dependency-as-one-error
artifact: context
---

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
