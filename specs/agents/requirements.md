---
spec: agents.spec.md
---

## User Stories

- As a developer using Claude Code, I want `specsync agents install --claude` to add a `SKILL.md` and a `/specsync:create-spec` command so Claude auto-discovers spec-sync's workflow and I get a native slash command instead of relying on prose in CLAUDE.md
- As a developer using Cursor, I want the same skill + `/specsync-create-spec` command installed in Cursor's own conventions (flat filename, no frontmatter)
- As a developer using Codex CLI, I want a project-scoped `SKILL.md` installed without spec-sync writing anything outside my project directory
- As a developer using Gemini CLI, I want both a skill and a `/specsync:create-spec` TOML command installed
- As a developer, I want `/specsync:create-spec <name>` to scaffold a full spec (with companion files) by default, and `--minimal` to opt out
- As a developer, I want to pass a natural-language feature description instead of a bare module name (e.g. `/specsync:create-spec "I want a feature that lets users export their data as CSV"`) and have the agent pick a module name and draft the spec's Purpose/Requirements from it
- As a developer, I want `specsync agents install` with no target flags to install all four tools at once
- As a developer, I want `specsync agents status` to show which tools are installed
- As a developer, I want `specsync agents uninstall` to cleanly remove only spec-sync's own files, never touching unrelated commands a tool's shared directory might contain

## Acceptance Criteria

- Four tools supported: Claude, Cursor, Codex, Gemini
- Claude, Cursor, and Gemini each get a skill and a `create-spec` command; Codex gets a skill only
- Installation is idempotent per-artifact — re-installing an already-installed artifact is a no-op
- Uninstall never removes a tool's shared `commands/` directory, only spec-sync's own namespaced subdirectory/file within it
- Empty targets list means "all tools"
- `cmd_install` exits with code 1 if any tool installation fails

## Constraints

- Must never write outside the project root (no `~/.codex/prompts/`-style global writes), even where other ecosystem tools (e.g. OpenSpec) do
- No `toml` crate dependency exists in this project — Gemini's TOML command file must be a hand-built string template
- Must not assume undocumented behavior of a tool's command/skill format — verify against the tool's actual documented mechanics before implementing

## Out of Scope

- Additional slash commands beyond `create-spec` (e.g. native `check`/`coverage`/`score` commands) — deferred until real usage of `create-spec` shows a need
- Extending native skill/command installation to Copilot or the generic `AGENTS.md` fallback — those remain served by `hooks.rs`'s prose-instruction mechanism
- Persisting which tools were selected in config so a bare `specsync agents install` remembers prior choices
