---
module: hooks
version: 14
status: stable
files:
  - src/hooks.rs
db_tables: []
tracks: [39]
depends_on:
  - specs/util/util.spec.md
---

# Hooks

## Purpose

Manages agent instruction files and git hooks for spec-sync integration. Installs and uninstalls instruction snippets for Claude (CLAUDE.md), Cursor (.cursorrules), Copilot (.github/copilot-instructions.md), Agents (AGENTS.md), a git pre-commit hook, and Claude Code settings.json hooks.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `HookTarget` | All installable hook targets: Claude, Cursor, Copilot, Agents, Precommit, ClaudeCodeHook |

### Exported HookTarget Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `all` | — | `&'static [HookTarget]` | Returns slice of all hook targets |
| `name` | `&self` | `&'static str` | Short name string for this target (e.g. "claude", "precommit") |
| `description` | `&self` | `&'static str` | Human-readable description (e.g. "CLAUDE.md agent instructions") |
| `from_str` | `s: &str` | `Option<Self>` | Parse a hook target from string (case-insensitive, aliases supported) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `is_installed` | `root, target` | `bool` | Check if a specific hook target is already installed |
| `install_hook` | `root, target` | `Result<bool, String>` | Install a single hook target; returns `Ok(false)` if already present |
| `uninstall_hook` | `root, target` | `Result<bool, String>` | Uninstall a single hook target; returns `Ok(false)` if not found |
| `cmd_install` | `root, targets` | `()` | CLI handler: install specified targets (or all if empty) |
| `cmd_uninstall` | `root, targets` | `()` | CLI handler: uninstall specified targets (or all if empty) |
| `cmd_status` | `root` | `()` | CLI handler: show installation status of all hook targets |

## Invariants

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
14. Every file `install` and `uninstall` create, rewrite or remove (`CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md`, `.claude/settings.json`) is reached through `util`'s project file functions. A link that stays inside the project is written through, so `CLAUDE.md` as a link to `AGENTS.md` keeps working. A target, or a directory on the way to it, that leaves the project through a symbolic link, is a dangling link, or is not a regular file fails that target with an error naming the path, and both commands exit 1 (REQ-hooks-006).
15. Every read of a project file here, the hook files (8 MiB); the pre-commit hook is read through the hooks folder handle (invariant 16), goes through the bounded project read of `util`. A link that leaves the project, a dangling link, a file that is not regular and a file over its bound are refused and handled as that read's other read errors are, so nothing here waits on a FIFO or a device or reads past its bound (REQ-hooks-007).
16. The hooks folder Git names (`git rev-parse --git-path hooks`, which honors `core.hooksPath`) is reached one component at a time without following any symbolic link, from the canonical Git directory when it lies there, from the canonical work-tree top when it lies there (husky's `.husky/_`, a `.githooks` folder), and otherwise from the canonical parent of the person's own folder. A folder outside both is made, when missing, only when `git config --show-scope` says `core.hooksPath` comes from the `global` or `system` config; set in the repository's config, which an archive can ship, or anywhere else, a missing folder is an error and nothing is made there. `pre-commit` is read without following a link or blocking, checked on the handle and within 8 MiB; replaced through its open, checked handle; created only with an exclusive create; made executable through the handle; and removed by unlinking its name. A link on the way, or a linked or dangling `pre-commit`, is refused with the path, and nothing is written or made executable outside the folder (REQ-hooks-009).

## Behavioral Examples

### Scenario: Install all hooks

- **Given** a project with no hooks installed
- **When** `cmd_install(root, &[])` is called
- **Then** installs CLAUDE.md, .cursorrules, copilot-instructions.md, AGENTS.md, pre-commit hook, and Claude Code settings

### Scenario: Already installed

- **Given** CLAUDE.md already contains "Spec-Sync Integration"
- **When** `install_hook(root, HookTarget::Claude)` is called
- **Then** returns `Ok(false)` without modifying the file

### Scenario: Uninstall cursor rules

- **Given** .cursorrules contains the spec-sync section
- **When** `uninstall_hook(root, HookTarget::Cursor)` is called
- **Then** removes the spec-sync section, returns `Ok(true)`; deletes the file if it becomes empty

### Scenario: Check status

- **Given** Claude and Precommit hooks are installed, others are not
- **When** `cmd_status(root)` is called
- **Then** shows "installed" for Claude and Precommit, "not installed" for the rest

### Scenario: Shared hook directory

- **Given** two projects resolve to the same `core.hooksPath`
- **When** each installs its pre-commit integration
- **Then** each project receives a distinct managed block and uninstalling either leaves the other intact

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot read/write file | Returns `Err` with descriptive message |
| Cannot create directory | Returns `Err` with descriptive message |
| Uninstall Claude Code hook | Returns `Err` — must be removed manually |
| Cannot parse existing settings.json | Returns `Err` with parse error |
| Partial or reversed pre-commit managed markers | Returns `Err` without rewriting the hook |
| Project is not inside a Git repository | Pre-commit installation returns an actionable `Err` |
| A folder on the hooks path is a symbolic link, or `pre-commit` is a link, dangling or not | `Err` naming the path (`configured Git hooks directory is reached through a symlink ...`, `refusing to replace symlinked pre-commit hook`); nothing is created or made executable at a link's target |
| A hook target, or `.github` or `.claude`, leaves the project through a symbolic link, is a dangling link, or is not a regular file | That target fails with an error naming the path; nothing is written through the link or created at its target; `install` and `uninstall` exit 1 |
| A project file this module reads leaves the project through a link, dangles, is not a regular file, or is over its bound | Refused without waiting or reading past the bound, and handled like that read's other read errors |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| serde_json | JSON parsing for Claude Code settings.json |
| colored | Terminal output formatting |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `cmd_install`, `cmd_uninstall`, `cmd_status`, `HookTarget` |

## Change Log

| Date | Change |
|------|--------|
| 2026-11-01 | v12: write hook files without en or em dashes; the new begin sentinel uses a semicolon and uninstall still recognizes the 6.1.1 form (REQ-hooks-005, #827). |
| 2026-10-15 | Quote the generated failure hint as a fixed-format printf argument; execute installed hooks against ordinary and shell-metacharacter project paths on success and failure (#782). |
| 2026-10-10 | Add `hooks.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-03-25 | Initial spec |
| 2026-03-30 | Add Agents (AGENTS.md) hook target |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-09-09 | close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization: Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs |
| 2026-10-10 | tell-agents-to-record-lessons-in-the-module-3md-file: Tell agents to record lessons in the module 3md file |
| 2026-10-16 | keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics: Keep project paths literal in generated pre-commit failure diagnostics |
| 2026-11-01 | write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier: Write generated Markdown and hook files without em or en dashes while still recognizing the managed markers earlier versions wrote |
| 2026-11-05 | stop-waiting-on-a-descendant-pipe-after-the-git-child-exits: Stop waiting on a descendant pipe after the git child exits |
