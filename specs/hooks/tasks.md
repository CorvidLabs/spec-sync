---
spec: hooks.spec.md
---

## Tasks

- [ ] Add Claude Code hook uninstallation (currently refused as too risky)
- [ ] Add Windsurf/Cline agent instruction support
- [ ] Add `--check` flag to verify hook content is up-to-date (not just installed)
- [ ] Support custom hook content via config (allow projects to override default instructions)

## Done

- [x] Claude (CLAUDE.md) hook install/uninstall/status
- [x] Cursor (.cursorrules) hook install/uninstall/status
- [x] Copilot (.github/copilot-instructions.md) hook install/uninstall/status
- [x] Pre-commit (.git/hooks/pre-commit) hook install/uninstall
- [x] Claude Code settings.json hook install
- [x] Idempotent installation with marker string detection
- [x] Safe append to existing files
- [x] All-targets default when no flags specified

## Gaps

- No hook content versioning — if the built-in instruction text changes, existing installations won't know they're outdated
- Claude Code hook uninstall is a manual process

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
