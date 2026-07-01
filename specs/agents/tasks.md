---
spec: agents.spec.md
---

## Tasks

- [ ] Verify inside a real Claude Code / Cursor / Codex / Gemini CLI session that the installed artifacts are actually auto-discovered/listed as documented
- [ ] Consider native `check`/`coverage`/`score` slash commands if `create-spec` sees real usage and demand shows up
- [ ] Consider persisting selected tools in `.specsync/config.toml` so a bare `specsync agents install` remembers prior choices

## Done

- [x] Claude Code skill + `/specsync:create-spec` command install/uninstall/status
- [x] Cursor skill + `/specsync-create-spec` command install/uninstall/status (flat filename, no frontmatter)
- [x] Codex CLI skill install/uninstall/status (project-scoped, no command)
- [x] Gemini CLI skill + `/specsync:create-spec` TOML command install/uninstall/status
- [x] `create-spec` command accepts a bare module name or a natural-language feature description
- [x] `--minimal` flag support (spec-only via `specsync new`, vs. full scaffold via `specsync scaffold`)
- [x] Idempotent per-artifact installation
- [x] Safe uninstall — never touches a tool's shared `commands/` directory or unrelated sibling files
- [x] All-tools default when no flags specified

## Gaps

- No artifact content versioning — if the built-in SKILL.md/command body text changes, existing installations won't know they're outdated (same known gap as `hooks.rs`)
- Cannot verify real-world auto-discovery behavior inside actual Claude Code/Cursor/Codex/Gemini CLI sessions from this environment

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
