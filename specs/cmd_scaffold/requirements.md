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
- Both commands delegate rendering to the generator: no source emits explicit `files: []`, and detected exports populate Public API rows
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

The scaffold command SHALL create a validated full spec workspace and register it without overwriting existing canonical files.

Acceptance Criteria
- `cmd_add_spec` writes `<specs_dir>/<module>/<module>.spec.md` from the built-in template and never uses AI
- `cmd_scaffold` resolves the target specs directory from the `--dir` argument when provided, otherwise from `config.specs_dir`
- When a `--template` directory is provided, both the spec body and companions are produced from that template; otherwise the built-in generator is used
- Both commands auto-detect source files for the module: `cmd_add_spec` walks `<source_dir>/<module>/` matching `source_extensions`; `cmd_scaffold` delegates to `generator::find_files_for_module`
- Both commands delegate rendering to the generator: no source emits explicit `files: []`, and detected exports populate Public API rows
- If the spec file already exists, neither command overwrites it; both still backfill any missing companion files and return early
- Companion files always include tasks.md, context.md, requirements.md, testing.md; design.md is generated only when `config.companions.design` is true
- `cmd_scaffold` registers the new module in `specsync-registry.toml` only when that file already exists at the repo root
- On success, paths are printed relative to `root` with a checkmark; auto-detected source counts are reported

### REQ-cmd-scaffold-002

The add-spec scaffold SHALL exclude recognized test files from auto-detected module sources.

Acceptance Criteria

- JavaScript-family `.test.*` and `.spec.*` files are omitted.
- Production files with configured or default source extensions remain included.
