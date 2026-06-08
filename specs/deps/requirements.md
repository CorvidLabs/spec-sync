---
spec: deps.spec.md
---

## User Stories

- As a developer, I want spec-sync to verify that every `depends_on` reference points to a module that actually has a spec so that stale or typo'd dependency links are caught
- As a maintainer, I want circular dependency chains between specs reported so that I can keep the module graph acyclic
- As a developer, I want imports in my source code cross-checked against declared `depends_on` so that undeclared cross-module coupling surfaces as a warning
- As a developer working across projects, I want cross-project dependency refs skipped during local validation so that they aren't flagged as missing

## Acceptance Criteria

- `build_dep_graph` parses every spec's frontmatter, keying nodes by `module` name, recording `depends_on` (as module names via `extract_module_from_dep_path`) and `files`
- `extract_module_from_dep_path` resolves `specs/<m>/<m>.spec.md` and bare module names to `<m>`, returning `None` for unrelated paths
- Cross-project refs (those matching `is_cross_project_ref`) are excluded from declared deps and never reported as missing
- `validate_deps` reports an error for each declared dependency whose target module has no spec, populating both `errors` and `missing_deps`
- `detect_cycles` finds all circular chains via DFS coloring; each cycle is reported as a `Circular dependency: a -> b -> a` error
- `extract_imports` extracts imported module names for Rust (`use`/`mod`), TypeScript/JavaScript (`import ... from`/`require`), and Python (`import`/`from .`); unsupported languages return an empty set
- `check_undeclared_imports` warns only when a source import matches a known spec module, is not already declared, and is not the module's own name (self-imports are never flagged)
- `topological_sort` returns a deterministic order for a DAG and `None` when the graph contains a cycle

## Constraints

- Import extraction is regex-based (`LazyLock<Regex>`), best-effort, and does not run a real parser
- Unreadable source files and unparseable spec frontmatter are skipped silently, not treated as errors
- Only Rust, TypeScript, and Python imports are analyzed; other languages contribute no import edges
- Must not panic on malformed input — returns a populated `DepsReport` regardless

## Out of Scope

- Resolving or validating cross-project dependency references (skipped locally)
- Suggesting or auto-adding missing `depends_on` entries
- Transitive/version-aware dependency resolution
- Import analysis for languages beyond Rust, TypeScript, and Python
