# Preserve generated agent arguments

## ADDED

### REQUIREMENT REQ-agents-002

Generated agent integrations SHALL preserve complete user intent across each tool's documented argument syntax.

Acceptance Criteria

- Create-spec guidance removes supported flags before classifying the complete remaining input.
- A complete single module identifier is preserved unchanged.
- Quoted or unquoted natural-language descriptions are classified before a kebab-case module name is derived.
- Gemini create-change guidance uses `{{args}}` and contains no `$ARGUMENTS` reference.
- Every generated skill and create-change command quotes a free-text interview answer as one positional argument.
- Reinstalling all four integrations remains deterministic and idempotent.

## MODIFIED

### SPEC SECTION Invariants

1. Each `AgentTool` owns an SDD skill and, where supported, both `create-spec` and `create-change` commands — Codex has the project skill only because its command mechanism is deprecated/global.
2. Installation is idempotent per-artifact and content-aware — `install_agent` writes an artifact when it's missing *or* when its existing content differs from the current template (so upgrading spec-sync refreshes stale installations), and returns `Ok(false)` only when every artifact already matches the current template exactly.
3. Every artifact spec-sync writes lives inside a `spec-sync/`-named skill folder or a `specsync`-namespaced command file/directory that spec-sync fully owns — no marker-string surgery on shared files is needed (unlike `hooks.rs`).
4. `uninstall_agent` removes the skill directory wholesale (`remove_dir_all`) and the command file, then removes the command file's immediate parent directory only if that parent is named `specsync` and is now empty — it never removes a tool's shared `commands/` directory (e.g. `.claude/commands/`, `.cursor/commands/`), which may hold unrelated user commands.
5. Cursor's command file is flat (`.cursor/commands/specsync-create-spec.md`, no namespaced subdirectory, no YAML frontmatter) since Cursor's command mechanism doesn't support either.
6. Claude and Gemini's commands live in a namespaced subdirectory (`.claude/commands/specsync/create-spec.md`, `.gemini/commands/specsync/create-spec.toml`) so they're invoked as `/specsync:create-spec`.
7. Gemini's command file is TOML (`description`/`prompt` keys, `{{args}}` placeholder), hand-built as a string template since no `toml` crate dependency exists in this project.
8. Empty targets list means "all tools", matching the `hooks` module's convention.
9. `cmd_install` exits with code 1 if any tool installation fails.
10. Generated create-spec and create-change assets preserve complete arguments using each tool's native placeholder and quote free-text interview answers as one CLI argument.
