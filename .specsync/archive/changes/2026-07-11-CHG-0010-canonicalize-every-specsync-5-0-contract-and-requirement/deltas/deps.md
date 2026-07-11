## ADDED

### REQUIREMENT REQ-deps-001

The dependency module SHALL build and validate a deterministic module graph including missing references, cycles, and undeclared source imports.

Acceptance Criteria
- `build_dep_graph` parses every spec's frontmatter, keying nodes by `module` name, recording `depends_on` (as module names via `extract_module_from_dep_path`) and `files`
- `extract_module_from_dep_path` resolves `specs/<m>/<m>.spec.md` and bare module names to `<m>`, returning `None` for unrelated paths
- Cross-project refs (those matching `is_cross_project_ref`) are excluded from declared deps and never reported as missing
- `validate_deps` reports an error for each declared dependency whose target module has no spec, populating both `errors` and `missing_deps`
- `detect_cycles` finds all circular chains via DFS coloring; each cycle is reported as a `Circular dependency: a -> b -> a` error
- `extract_imports` extracts imported module names for Rust (`use`/`mod`), TypeScript/JavaScript (`import ... from`/`require`), and Python (`import`/`from .`); unsupported languages return an empty set
- Rust extraction ignores comments and ordinary, raw, byte, and raw-byte string literals.
- Rust top-level module names resolve to the canonical spec owning `src/<module>.rs` or
  `src/<module>/mod.rs` before undeclared-edge comparison.
- `check_undeclared_imports` warns only when a source import matches a known spec module, is not already declared, and is not the module's own name (self-imports are never flagged)
- `topological_sort` returns a deterministic order for a DAG and `None` when the graph contains a cycle
