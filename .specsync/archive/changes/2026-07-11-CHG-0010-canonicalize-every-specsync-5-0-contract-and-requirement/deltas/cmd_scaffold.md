## ADDED

### REQUIREMENT REQ-cmd-scaffold-001

The scaffold command SHALL create a validated full spec workspace and register it without overwriting existing canonical files.

Acceptance Criteria
- `cmd_add_spec` writes `<specs_dir>/<module>/<module>.spec.md` from the built-in template and never uses AI
- `cmd_scaffold` resolves the target specs directory from the `--dir` argument when provided, otherwise from `config.specs_dir`
- When a `--template` directory is provided, both the spec body and companions are produced from that template; otherwise the built-in generator is used
- Both commands auto-detect source files for the module: `cmd_add_spec` walks `<source_dir>/<module>/` matching `source_extensions`; `cmd_scaffold` delegates to `generator::find_files_for_module`
- If the spec file already exists, neither command overwrites it; both still backfill any missing companion files and return early
- Companion files always include tasks.md, context.md, requirements.md, testing.md; design.md is generated only when `config.companions.design` is true
- `cmd_scaffold` registers the new module in `specsync-registry.toml` only when that file already exists at the repo root
- On success, paths are printed relative to `root` with a checkmark; auto-detected source counts are reported
