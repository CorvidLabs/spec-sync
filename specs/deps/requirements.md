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

### REQ-deps-001

The `deps` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-deps-002

Dependency analysis SHALL report every spec it dropped, and SHALL NOT declare a graph valid
when any declaration could not be read.

Acceptance Criteria
- Frontmatter errors surfaced by the parser are reported rather than discarded, using the same wording the validator emits.
- A spec whose frontmatter cannot be parsed, and a spec declaring no module, are each reported as excluded from the analysis.
- Any such report is an error, so the command exits non-zero and does not print that all declarations are valid.
- A project whose specs are all well-formed continues to report a valid graph and exit zero.

### REQ-deps-003

Dependency analysis SHALL NOT report a clean graph built from imports it could not read or could not resolve.

Acceptance Criteria
- Kotlin imports resolve against a package topology built first from each JVM file's own `package` declaration and then from directory layout, declaration winning, so a file whose directory does not mirror its package still produces an edge.
- Every imported package resolves to exactly one of: owned by a spec module, foreign to every namespace the project occupies, or inside the project's namespace but unattributed.
- An unattributed import is recorded rather than dropped. When nothing is known about the project's packages, an unowned import is unattributed rather than foreign, so silence is never the default.
- A package claimed by two modules is left unowned and disclosed rather than guessed.

### REQ-deps-004

`deps` SHALL read spec files and the source files it reads for imports only through the bounded project read of `util` (REQ-util-003), with a bound of 8 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-deps-005

`deps` SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). It SHALL find spec files only through the confined, budgeted walk of `validator` (REQ-validator-050), so a `specs` link that leaves the repository, or a tree past the walk's budget, stops it with the reason instead of being walked.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source
- A `specs` link to an outside directory, and a tree deeper than the budget, stop `deps` with the reason.

