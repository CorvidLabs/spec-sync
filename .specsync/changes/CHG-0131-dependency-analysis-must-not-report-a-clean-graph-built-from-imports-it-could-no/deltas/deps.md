## ADDED

### REQUIREMENT REQ-deps-003

Dependency analysis SHALL NOT report a clean graph built from imports it could not read or could not resolve.

Acceptance Criteria
- Kotlin imports resolve against a package topology built first from each JVM file's own `package` declaration and then from directory layout, declaration winning, so a file whose directory does not mirror its package still produces an edge.
- Every imported package resolves to exactly one of: owned by a spec module, foreign to every namespace the project occupies, or inside the project's namespace but unattributed.
- An unattributed import is recorded rather than dropped. When nothing is known about the project's packages, an unowned import is unattributed rather than foreign, so silence is never the default.
- A package claimed by two modules is left unowned and disclosed rather than guessed.

## MODIFIED

### SPEC SECTION Public API

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
| `extract_imports` | `file_path, content` | `HashSet<String>` | Extract imports from source (Rust, TypeScript, Python, Kotlin). Rust/TS/Python yield module tokens; Kotlin yields package paths such as `com.example.core` that validation resolves to owning spec modules. Empty set for any other language |
| `format_report` | `report: &DepsReport` | `String` | Format dependency report as colored terminal text |
| `topological_sort` | `graph: &HashMap<String, DepNode>` | `Option<Vec<String>>` | Topologically sort modules; returns None if cycles exist |
| `unanalyzed_languages_note` | `report: &DepsReport` | `Option<String>` | One sentence naming the languages whose imports went unread, or `None`. Fires only for languages that HAVE an import construct — a YAML file has no imports to miss |
| `unresolved_imports_note` | `report: &DepsReport` | `Option<String>` | One sentence naming imports that were read but could not be mapped to a spec module, or `None`. This is the state the previous fix dropped silently, which is why it is a reported outcome rather than an absent one |
| `valid_declarations_line` | `report: &DepsReport` | `&'static str` | The success sentence, qualified when languages went unread and/or imports went unattributed, so a clean verdict never overstates what was analysed |

