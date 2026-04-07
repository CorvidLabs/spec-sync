---
module: deps
version: 1
status: stable
files:
  - src/deps.rs
db_tables: []
tracks: [139]
depends_on:
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Deps

## Purpose

Cross-module dependency validation. Parses `depends_on` declarations from spec frontmatter, builds a dependency graph, validates that declared dependencies exist, detects circular dependency chains, and cross-references declared dependencies against actual import statements in source code (Rust, TypeScript, Python).

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `DepNode` | A node in the dependency graph with module name, spec path, declared deps, and source files |
| `DepsReport` | Validation result with errors, warnings, module count, edge count, and circular chains |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `build_dep_graph` | `root, specs_dir` | `HashMap<String, DepNode>` | Parse all specs and build the dependency graph |
| `validate_deps` | `root, specs_dir` | `DepsReport` | Full dependency validation: missing deps, cycles, undeclared imports |
| `extract_imports` | `file_path, content` | `HashSet<String>` | Extract import/use statements from source code (Rust, TypeScript, Python) |
| `format_report` | `report: &DepsReport` | `String` | Format dependency report as colored terminal text |
| `topological_sort` | `graph: &HashMap<String, DepNode>` | `Option<Vec<String>>` | Topologically sort modules; returns None if cycles exist |

## Invariants

1. Import extraction uses language-specific regex: Rust (`use crate::`, `mod`), TypeScript (`import from`, `require`), Python (`import`, `from .module`)
2. Circular dependency detection traverses the full graph — reports all cycles, not just the first
3. `topological_sort` returns `None` when cycles are present — does not partial-sort
4. Cross-project refs (containing `/`) in `depends_on` are skipped — only local deps are validated
5. Undeclared imports (found in source but not in `depends_on`) are reported as warnings, not errors
6. Module names in `depends_on` paths are extracted from the path's directory component

## Behavioral Examples

### Scenario: Detect missing dependency

- **Given** spec A declares `depends_on: [specs/nonexistent/nonexistent.spec.md]`
- **When** `validate_deps` is called
- **Then** report contains an error about the missing dependency spec

### Scenario: Detect circular dependency

- **Given** spec A depends on B and spec B depends on A
- **When** `validate_deps` is called
- **Then** report's `cycles` field contains the chain `[A, B, A]`

### Scenario: Extract Rust imports

- **Given** a file containing `use crate::config::load_config;`
- **When** `extract_imports(path, content)` is called
- **Then** returns a set containing `"config"`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Source file unreadable | Skipped during import extraction |
| Spec frontmatter unparseable | Module excluded from dependency graph |
| No specs found in specs_dir | Returns empty graph and clean DepsReport |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` |
| types | `Language` |
| validator | `find_spec_files`, `is_cross_project_ref` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli | `validate_deps`, `build_dep_graph`, `format_report`, `topological_sort` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-07 | Initial spec |
