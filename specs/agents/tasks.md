---
spec: agents.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Consider persisting selected tools in `.specsync/config.toml` so a bare `specsync agents install` remembers prior choices
- [ ] Consider native `check`/`coverage`/`score` slash commands if `create-spec` sees real usage and demand shows up

## Manual/Environment Validation

- [ ] Verify inside a real Claude Code / Cursor / Codex / Gemini CLI session that the installed artifacts are actually auto-discovered/listed as documented

## Done

- [x] Claude Code skill + `/specsync:create-spec` command install/uninstall/status
- [x] Cursor skill + `/specsync-create-spec` command install/uninstall/status (flat filename, no frontmatter)
- [x] Codex CLI skill install/uninstall/status (project-scoped, no command)
- [x] Gemini CLI skill + `/specsync:create-spec` TOML command install/uninstall/status
- [x] `create-spec` command accepts a bare module name or a natural-language feature description
- [x] `--minimal` flag support (spec-only via `specsync new`, vs. full scaffold via `specsync scaffold`)
- [x] Idempotent per-artifact installation
- [x] Content-aware reinstall — `install_agent` overwrites artifacts whose content has drifted from the current template, so upgrading spec-sync refreshes stale installations instead of leaving them outdated
- [x] Safe uninstall — never touches a tool's shared `commands/` directory or unrelated sibling files
- [x] All-tools default when no flags specified
- [x] Verified SDD lifecycle instructions for all four agent skills
- [x] Native create-change commands for Claude, Cursor, and Gemini
- [x] Human approval safeguards and deterministic JSON interview guidance

## Gaps

- Cannot verify real-world auto-discovery behavior inside actual Claude Code/Cursor/Codex/Gemini CLI sessions from this environment

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
