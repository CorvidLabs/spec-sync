---
spec: deps.spec.md
---

## Key Decisions

- **Module-name keyed graph**: nodes are keyed by frontmatter `module`, and `depends_on` paths are reduced to module names via `extract_module_from_dep_path` so cycle/missing checks compare bare names.
- **DFS coloring for cycles**: `detect_cycles` uses White/Gray/Black coloring; a back-edge to a Gray node yields a reported chain. All cycles are collected, not just the first.
- **Regex import extraction**: `extract_imports` uses cached `LazyLock<Regex>` patterns per language (Rust `use`/`mod`, TS `import`/`require`, Python `import`/`from .`, Kotlin/JVM `import` with package attribution). It is best-effort, not a real parser.
- **Conservative warnings**: undeclared-import warnings only fire when the imported name is a known spec module, isn't already declared, and isn't the module itself — keeping false positives low.
- **Disclosed skips**: unreadable source files and unparseable frontmatter are dropped from the graph, but each one is recorded as a hard error on the report (#550) — they were once silent, and `deps` then affirmed "all dependency declarations are valid" over specs it could not read. Validation still returns a populated `DepsReport` rather than erroring out.
- **Cross-project refs skipped**: refs matching `is_cross_project_ref` are dropped before validation so external links aren't reported as missing.

## Key Files

- `src/deps.rs` - Graph construction, cycle detection, import analysis, topological sort, and inline tests
- `src/validator.rs` - provides `find_spec_files` and `is_cross_project_ref` used here
- `src/commands/deps.rs` - wires the `specsync deps` subcommand to `validate_deps`
- `specs/deps/deps.spec.md` - Module specification

## Current Status

Module is stable and complete, with extensive inline unit tests. Validation is exposed via the `specsync deps` subcommand.
