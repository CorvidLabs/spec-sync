---
spec: cmd_scaffold.spec.md
---

## User Stories

- As a developer, I want `specsync add-spec <module>` to create a ready-to-edit spec from a built-in template with my module's source files already detected so I can start documenting immediately
- As a developer, I want `specsync scaffold <module>` to support a custom specs directory and a custom template directory so I can match an existing project layout or house style
- As a team lead, I want scaffolding to auto-register a new module in `specsync-registry.toml` when a registry exists so the module is tracked without a manual edit
- As any user, I want companion files (tasks/context/requirements/testing, plus design when enabled) generated alongside the spec so the documentation set is complete from the start

## Acceptance Criteria

- `cmd_add_spec` writes `<specs_dir>/<module>/<module>.spec.md` from the built-in template and never uses AI
- `cmd_scaffold` resolves the target specs directory from the `--dir` argument when provided, otherwise from `config.specs_dir`
- When a `--template` directory is provided, both the spec body and companions are produced from that template; otherwise the built-in generator is used
- Both commands auto-detect source files for the module: `cmd_add_spec` walks `<source_dir>/<module>/` matching `source_extensions`; `cmd_scaffold` delegates to `generator::find_files_for_module`
- If the spec file already exists, neither command overwrites it; both still backfill any missing companion files and return early
- Companion files always include tasks.md, context.md, requirements.md, testing.md; design.md is generated only when `config.companions.design` is true
- `cmd_scaffold` registers the new module in `specsync-registry.toml` only when that file already exists at the repo root
- On success, paths are printed relative to `root` with a checkmark; auto-detected source counts are reported

## Constraints

- Must not panic on expected error conditions — print to stderr and `process::exit(1)`
- Directory-creation and file-write failures exit with code 1 and a clear message
- Read-only with respect to existing specs: an existing `*.spec.md` is never modified
- Source-file paths are normalized to forward slashes (`\` → `/`) for cross-platform manifests

## Out of Scope

- Coding-agent enrichment after deterministic scaffolding (handled through installed agent skills or MCP)
- Interactive prompts (handled by the `wizard` command)
- Editing or regenerating an existing spec's body
- Validation or scoring of the generated spec

### REQ-cmd-scaffold-001

The `cmd_scaffold` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-scaffold-002

The add-spec scaffold SHALL exclude recognized test files from auto-detected module sources.

Acceptance Criteria

- JavaScript-family `.test.*` and `.spec.*` files are omitted.
- Production files with configured or default source extensions remain included.


### REQ-cmd-scaffold-003

`scaffold` SHALL derive exports from the configured surface and parse mode.

Acceptance Criteria
- The Public API it writes contains only symbols `check` will accept.

### REQ-cmd-scaffold-004

`scaffold --dir` SHALL remain beneath the project root. Absolute paths outside the root and `..` components are refused.

Acceptance Criteria
- `--dir /tmp` exits 1 naming `scaffold --dir must remain beneath the project root`.
- `--dir ../escape` exits 1 via `confined_generation_path` and writes nothing outside the project.
- A relative directory under the root is joined to the root and used as the specs directory.

### REQ-cmd-scaffold-005

Add-spec and scaffold SHALL bind exact configured root Cargo source files through shared source selection.

Acceptance Criteria
- Unambiguous root-file projects emit nonempty truthful file lists and validate; existing companion no-overwrite behavior, explicit mappings, filters and ambiguity remain intact.

### REQ-cmd-scaffold-006

`scaffold` SHALL exclude files already claimed by an existing spec's `files:` list from the scaffolded spec's `files:` list.

Acceptance Criteria
- `scaffold foo` where `src/foo.rs` is already listed by another spec never lists `src/foo.rs`.
- A module with no unowned candidate files takes the existing no-files path: the spec is written with `files: []` and the empty-files warning.

### REQ-cmd-scaffold-007

`scaffold` and `add-spec` SHALL create the spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010), and `scaffold` SHALL print the error and exit 1 when the registry refuses the append (REQ-registry-003). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- A dangling link at `specs/<name>/<name>.spec.md`: `scaffold <name>` and `add-spec <name>` exit 1 naming the path and nothing is created at the link's target.
- `specs/<name>` linked to a directory outside the project: both exit 1 and nothing is created there.
- `.specsync/registry.toml` linked to a file outside the project: `scaffold <name>` exits 1 naming the registry and the outside file is unchanged.
- `specs/<name>` linked to a directory inside the project: the spec is created in that directory.

### REQ-cmd-scaffold-008

`scaffold` SHALL check a new module name for a case collision only against spec folders the project rule reaches (REQ-commands-016).

Acceptance Criteria
- `scaffold auth` with a `specs` link out holding `Auth` reports no collision from outside.

