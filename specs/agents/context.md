---
spec: agents.spec.md
---

## Key Decisions

- **Sibling to `hooks.rs`, not a variant of it**: `hooks.rs` appends prose instructions to shared files a user may already have content in (CLAUDE.md, .cursorrules, AGENTS.md), requiring marker-string detection and surgical section removal. This module writes brand-new files/directories spec-sync fully owns, so install/uninstall are plain existence checks and whole-directory/file operations — a meaningfully simpler ownership model, done deliberately rather than forced into `HookTarget`'s shape.
- **Per-tool capability matrix, not one enum arm per artifact**: `AgentTool` has 4 variants (not `ClaudeSkill`/`ClaudeCommand`/... as separate variants) because the natural unit of user control is "the tool", e.g. `--claude` should install both its skill and command in one shot. `skill_dir()`/`command_path()` return `Option<PathBuf>` per tool since Codex has no command and (originally) Gemini had no skill.
- **Gemini skill added after initial ship**: Gemini CLI added stable `SKILL.md` support in 2026 — verified against `geminicli.com` docs and the `google-gemini/gemini-cli` repo — after this module's PR was already up. Gemini went from command-only to skill+command in a follow-up commit.
- **Codex stays project-scoped, diverging from OpenSpec**: OpenSpec's own adapter (`Fission-AI/OpenSpec`, `src/core/command-generation/adapters/codex.ts`) writes Codex's command globally to `$CODEX_HOME/prompts/opsx-<id>.md` (outside any project). This module deliberately does not — Codex gets a project-local skill only, consistent with every other spec-sync integration never writing outside the project root, and consistent with OpenAI's own guidance that the global prompts mechanism is deprecated in favor of skills.
- **Cursor has no command frontmatter, verified independently**: OpenSpec's own `cursorAdapter` writes YAML frontmatter (`name`/`id`/`category`/`description`) into Cursor command files, but independent web research confirmed Cursor's command mechanism has no frontmatter support at all — the filename alone determines the command name. This module's Cursor command is plain markdown, no frontmatter block.
- **SKILL_BODY is a standalone copy, not shared with `hooks.rs`**: `hooks.rs`'s four instruction snippets (CLAUDE_MD_SNIPPET, CURSORRULES_SNIPPET, COPILOT_INSTRUCTIONS_SNIPPET, AGENTS_MD_SNIPPET) aren't identical to each other, so unifying all of them with this module's `SKILL_BODY` was left out of scope to avoid changing already-shipped, tested output for an unrelated reason.

## Files to Read First

- `src/agents.rs` — the whole module: `AgentTool` enum, path routing, install/uninstall/status, skill and command content builders.
- `src/hooks.rs` — the sibling prose-instruction system this module is distinct from; useful for contrast, not reuse.
- `src/commands/agents.rs` — thin CLI dispatcher (see `cmd_agents` spec).

## Current Status

Updated for 5.0 SDD. All four tools receive the verified lifecycle skill; Claude, Cursor, and Gemini also receive four commands — create-spec, create-change, check, and audit (`command_paths()` returns all four). Installation remains content-aware and safe for shared command directories, with unit coverage for upgrade, idempotency, and uninstall behavior.

## Notes

- Generated SDD skill text tells the agent the scoped reviewer MAY be the definition approver. Do not invent a second identity to satisfy SpecSync; GitHub required reviews are the two-person gate when a repository wants one.
- The `create-spec` command's prompt body instructs the agent to parse either a bare module name or a natural-language feature description out of its arguments — that parsing happens in the agent's own reasoning, not in Rust code, since these are prompt files, not real CLI arg parsers.
- Gemini's `.toml` command file is a hand-built string template (`gemini_create_spec_toml()`), not a serialized struct. That predates the `toml` crate, which this project has depended on since #483; nothing has migrated it.
- Shared create-spec templates remove standalone `--minimal` flags before classifying the complete remaining input. The repository's Claude, Cursor, and Gemini command assets are regenerated from those templates and parity-tested byte for byte so contributors cannot receive stale first-token guidance.
