---
change: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
artifact: design
---

# Design

The public facade retains DepNode, DepsReport and re-exports the existing function paths. Private graph owns graph construction, validation, cycle detection and topological ordering. Private imports owns every regex, ImportDialect, lexical scanner, Rust module ownership, JVM package ownership and undeclared-import analysis. Private report owns disclosure notes and terminal rendering. Test files are grouped by their owner; the two fixture builders are shared only under cfg(test).

No public helper expansion is needed. Sibling cooperation uses minimal pub(super) seams, and helpers remain private wherever possible. Each production and test file stays below 1000 lines.
