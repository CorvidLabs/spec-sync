---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: design
---

# Design

- `fix_near_miss_headers` keeps the original Public API slice as the replacement buffer. `blank_fenced_code` is scan-only (header discovery). Replacements apply by blanked-copy offsets into the original, end-to-start, so a length-changing rename (`### Functions` → `### Exported Functions`) does not shift later matches. A fenced `### Exportd Functions` stays quoted text because it is spaces in the scan copy.
- `git_cmd` keeps `env_clear` + the PATH/locale/home/temp allowlist + `GIT_TERMINAL_PROMPT=0` / `GIT_OPTIONAL_LOCKS=0` / `LC_ALL=C`. It does **not** set `GIT_CEILING_DIRECTORIES`. Inherited ceiling is already wiped by `env_clear`. Nested-project walk-up is required. Snapshot isolation is the caller's job (place the snapshot outside any repo).
- `cmd_new` spec body is synchronized to the implementation already on this PR: `generate_spec`, `validate_scaffold_module_name`, `check_case_collision`, `collect_exports_for_files` / `generate_companion_files_for_spec`. No `src/commands/new.rs` change.
