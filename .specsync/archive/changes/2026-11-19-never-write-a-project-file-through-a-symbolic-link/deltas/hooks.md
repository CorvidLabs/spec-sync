## ADDED

### REQUIREMENT REQ-hooks-006

`hooks install` and `hooks uninstall` SHALL create, rewrite and remove `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md` and `.claude/settings.json` only through the project file functions of `util` (REQ-util-002). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- `CLAUDE.md` linked to a file outside the project: `hooks install --claude` fails naming `CLAUDE.md` and the outside file is unchanged. The same holds for `.cursorrules`, `AGENTS.md`, `.github/copilot-instructions.md` and `.claude/settings.json`.
- A dangling `CLAUDE.md` link creates nothing at its target.
- A `CLAUDE.md` link to a second link that leaves the project is refused.
- `.github` or `.claude` linked to a directory outside the project: nothing is created there.
- `CLAUDE.md` linked to `AGENTS.md` in the same project: `hooks install --claude` appends the block to `AGENTS.md` and `CLAUDE.md` stays a link.
- `hooks uninstall --claude` with `CLAUDE.md` linked to an outside file that holds the block fails naming `CLAUDE.md` and leaves that file unchanged.
