## ADDED

### REQUIREMENT REQ-cli-004

The root CLI SHALL reject inherited verification re-entry before dispatching any lifecycle command handler.

Acceptance Criteria

- Explicit and default `check`, `change`, and `lifecycle` commands fail before handler-specific discovery, warnings, validation, or mutation.
- The process emits one contextual diagnostic and exits non-zero.
- Commands outside the lifecycle boundary preserve current dispatch behavior.

## MODIFIED

### SPEC SECTION Invariants

1. When no subcommand is given, `check` runs by default
2. `--root` defaults to the current working directory; the path is validated (must be an existing directory — otherwise an error is printed and the process exits 2) and canonicalized
3. `--strict` causes warnings to produce a non-zero exit code
4. `--require-coverage N` causes exit 1 if file coverage percent < N
5. `--json` switches all output to machine-readable JSON (no ANSI colors)
6. `cmd_init` is idempotent and never overwrites current or legacy project configuration
7. `cmd_init_registry` is idempotent — does nothing if `specsync-registry.toml` already exists
8. `cmd_add_spec` generates companion files even if the spec already exists
9. `cmd_generate` re-runs validation after generating new specs to include them in the summary
10. `cmd_resolve --remote` performs network calls; without the flag, cross-project refs are listed but not verified
11. `load_and_discover` filters out spec files starting with `_` (template files)
12. Exit codes: 0 = success, 1 = errors (or warnings in strict mode, or coverage below threshold)
13. `collect_hook_targets` with no flags set returns an empty vec, meaning "all targets"
14. `--fix` only adds exports not already documented in the spec (no duplicates)
15. `--fix` modifies spec files on disk — validation runs after fix so the fixed specs are re-checked
16. `--fix` with `--json` suppresses the human-readable fix summary but still writes the fix
17. `cmd_diff` shells out to `git diff --name-only <base>` to detect changed files
18. `cmd_diff` only reports specs whose `files:` frontmatter list intersects the changed file set
19. `cmd_scaffold` auto-detects source files, creates companion files, and registers the module in `specsync-registry.toml` if it exists
20. `cmd_report` flags modules whose specs are N+ commits behind their source files (default threshold: 5)
21. `cmd_comment` without `--pr` prints the comment body to stdout; with `--pr N` posts via `gh` CLI
22. `cmd_changelog` requires a git ref range (e.g., `v0.1..v0.2`); exits 1 if range is invalid
23. `--enforcement` CLI flag overrides the effective loaded configuration (`.specsync/config.toml` first, with legacy compatibility fallbacks); `--strict` implies strict enforcement
24. Inherited verification context rejects `check`, `change`, and `lifecycle` before handler dispatch, while unrelated commands preserve their current behavior.
