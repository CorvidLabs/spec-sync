## MODIFIED

### SPEC SECTION Invariants

1. Installation is idempotent — re-installing an already-installed hook is a no-op returning `Ok(false)`
2. Agent instruction files are appended to existing files, not overwritten
3. Shared pre-commit files use balanced project-keyed managed markers; partial or reversed marker pairs fail before mutation
4. Pre-commit hook is made executable (mode 0o755) on Unix
5. Uninstalling Claude Code hook settings is refused — must be done manually (too risky to auto-edit)
6. Empty targets list means "all targets"
7. Pre-commit hook resolution honors Git worktrees, submodules, and `core.hooksPath`; installation inserts its blocking block (running `specsync check`, honoring configured `enforcement`) before a trailing `exit 0`
8. `cmd_install` exits with code 1 if any hook installation fails
9. Uninstall removes only the exact current-project block and preserves user and other-project content
10. The generated pre-commit hook runs `specsync check` without `--strict`, so first-run scaffold warnings do not block the first commit
11. Project directory names remain literal data in both the executable command
    and failure diagnostic. Spaces, apostrophes, dollar substitutions and
    backticks cannot alter shell syntax or execute directory-name content.
12. Instruction blocks, the pre-commit hook and Claude Code hook settings are written without U+2013 or U+2014. A new block begins with ``<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->``; uninstall also recognizes the begin sentinel releases up to and including 6.1.1 wrote, by an exact two-form list rather than a prefix. Project-keyed pre-commit blocks are found by their markers whatever body they hold; the unmarked legacy comparison still uses the current hook body, with no frozen copy of an older one
13. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
