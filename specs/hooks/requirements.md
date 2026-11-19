---
spec: hooks.spec.md
---

## User Stories

- As a developer using Claude Code, I want `specsync hooks install claude` to add spec-sync instructions to CLAUDE.md so that Claude automatically respects my specs when editing code
- As a developer using Cursor, I want spec-sync rules injected into .cursorrules so that Cursor follows spec conventions during code generation
- As a developer using Copilot, I want instructions added to .github/copilot-instructions.md so that Copilot suggestions align with documented APIs
- As a team lead, I want a pre-commit hook that runs `specsync check --strict` so that spec violations are caught before code reaches the remote
- As a developer, I want `specsync hooks install` with no targets to install all hooks at once so that setup is a single command
- As a developer, I want `specsync hooks status` to show which hooks are installed so that I can verify my setup
- As a developer, I want `specsync hooks uninstall` to cleanly remove hooks so that I can disable integration without manual file editing
- As a multi-agent team, I want AGENTS.md instructions installed so that any agent framework can discover spec-sync conventions

## Acceptance Criteria

- Generated pre-commit hooks treat the selected project directory as literal
  data in commands and diagnostics, preserving checker success/failure without
  executing shell syntax embedded in the directory name.

- Six targets supported: Claude, Cursor, Copilot, Agents, Precommit, ClaudeCodeHook
- Installation is idempotent — re-installing an already-installed hook is a no-op returning Ok(false)
- Agent instructions are appended to existing files, not overwritten
- Marker strings ("Spec-Sync Integration", "Spec-Sync Rules") are used to detect existing installations
- Pre-commit hook is made executable (0o755) on Unix systems
- Uninstalling Claude Code hook settings is refused (too risky to modify IDE settings)
- Empty targets list means "all targets"
- Pre-commit hook appends to existing hooks (preserves existing shebang and content)

## Constraints

- Must not overwrite user content in instruction files — only append spec-sync sections
- Pre-commit hook must be compatible with other hooks in the same file
- File permissions must be set correctly on Unix (executable bit for pre-commit)

## Out of Scope

- Installing hooks for AI tools not in the supported list
- Managing git hook frameworks (husky, lefthook, etc.)
- Modifying IDE settings beyond the Claude Code hook
- Auto-updating hook content when spec-sync is upgraded

### REQ-hooks-001

The `hooks` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-hooks-002

Hook installation SHALL resolve the repository's effective hook directory and manage only
project-keyed SpecSync blocks.

Acceptance Criteria

- Installation honors normal repositories, worktrees, submodules, and `core.hooksPath`.
- Managed blocks carry a stable project key so multiple projects sharing a hook path do not collide.
- Install is idempotent and the generated pre-commit hook runs `specsync check` (honoring `.specsync/config.toml` `enforcement`; default is strict on errors, warnings pass unless `--strict` is given) rather than hardcoding `--strict`.
- Uninstall removes only the matching SpecSync block and preserves user and other-project content.
- Symlink escapes, ambiguous hook roots, and unsafe paths fail before mutation.

### REQ-hooks-two-verb-001

Installed hooks instruction snippets SHALL document `change check` as scoped verification and `change audit` as active-workspace project health, and SHALL NOT instruct agents to treat check as full archive terminal-evidence validation.

Acceptance Criteria
- Claude/Agents.md-style snippets mention both verbs.
- Cursor/Copilot snippets mention the two-verb distinction.

### REQ-hooks-003

A first-run `init` → source file → `add-spec` → `check` (errors 0, warnings present) → `hooks install` → `git commit` sequence SHALL succeed without `--no-verify`, because the generated hook does not treat warnings as errors.

Acceptance Criteria
- The generated hook body contains `specsync check` or `specsync --root … check` and does not contain `check --strict`.
- The hook comment names `.specsync/config.toml`, not `specsync.json`, and does not call `warn` the default.
- `init_add_spec_hooks_install_then_commit_succeeds_without_strict` ends in a successful commit.

### REQ-hooks-004

Installed hook instruction snippets SHALL tell an agent that when `<module>.3md` exists beside the spec, a new lesson goes on a later plane of that file. The snippets SHALL still name `context.md` for a module that has no 3md file.

Acceptance Criteria
- Each hook snippet that describes finishing work mentions a later plane when `<module>.3md` exists.
- The snippets do not say that `context.md` is the only lesson file once `<module>.3md` exists.

### REQ-hooks-005

The system SHALL write hook instruction blocks, the Git pre-commit hook and Claude Code hook settings without U+2013 (en dash) or U+2014 (em dash), and SHALL keep recognizing the instruction-block begin sentinel that releases up to and including 6.1.1 wrote.

Acceptance Criteria
- `hooks install` for every target writes no U+2013 or U+2014.
- A new managed block begins with ``<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->`` and ends with the unchanged `<!-- specsync:hook:end -->`.
- `hooks uninstall` removes a block framed by the 6.1.1 begin sentinel exactly as it removes one framed by the new sentinel, preserving content before and after it.
- `hooks uninstall --precommit` removes a project-keyed managed block whatever body it holds, including the body 6.1.1 wrote, because the block is found by its markers.
- Unmarked legacy pre-commit recognition (an appended block after the legacy marker, or a whole-file hook) compares against the current hook body, as before; this requirement adds no frozen copy of an older body.
- The `Spec-Sync Integration` and `Spec-Sync Rules` detection markers, the end sentinel and the project-keyed pre-commit markers are unchanged.
- Installation does not rewrite a block that is already present.

### REQ-hooks-006

`hooks install` and `hooks uninstall` SHALL create, rewrite and remove `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md` and `.claude/settings.json` only through the project file functions of `util` (REQ-util-002). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- `CLAUDE.md` linked to a file outside the project: `hooks install --claude` fails naming `CLAUDE.md` and the outside file is unchanged. The same holds for `.cursorrules`, `AGENTS.md`, `.github/copilot-instructions.md` and `.claude/settings.json`.
- A dangling `CLAUDE.md` link creates nothing at its target.
- A `CLAUDE.md` link to a second link that leaves the project is refused.
- `.github` or `.claude` linked to a directory outside the project: nothing is created there.
- `CLAUDE.md` linked to `AGENTS.md` in the same project: `hooks install --claude` appends the block to `AGENTS.md` and `CLAUDE.md` stays a link.
- `hooks uninstall --claude` with `CLAUDE.md` linked to an outside file that holds the block fails naming `CLAUDE.md` and leaves that file unchanged.

### REQ-hooks-007

`hooks install`, `hooks uninstall` and `hooks status` SHALL read `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md` and `.claude/settings.json` only through the bounded project read of `util` (REQ-util-003), with a bound of 8 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound. The pre-commit hook in the Git hooks directory is not a project file and keeps its own guard.

Acceptance Criteria
- With `CLAUDE.md` a FIFO, `hooks status` finishes without waiting on it.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-hooks-008

`hooks` SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). Its pre-commit writes stay in the Git hooks directory `git rev-parse --git-path hooks` names.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source

