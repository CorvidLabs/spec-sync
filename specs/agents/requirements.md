---
spec: agents.spec.md
---

## User Stories

- As a developer using Claude Code, I want `specsync agents install --claude` to add a `SKILL.md` and a `/specsync:create-spec` command so Claude auto-discovers spec-sync's workflow and I get a native slash command instead of relying on prose in CLAUDE.md
- As a developer using Cursor, I want the same skill + `/specsync-create-spec` command installed in Cursor's own conventions (flat filename, no frontmatter)
- As a developer using Codex CLI, I want a project-scoped `SKILL.md` installed without spec-sync writing anything outside my project directory
- As a developer using Gemini CLI, I want both a skill and a `/specsync:create-spec` TOML command installed
- As a developer using Claude, Cursor, or Gemini, I want a native create-change command that starts the same deterministic verified-SDD interview as the CLI
- As a developer, I want `/specsync:create-spec <name>` to scaffold a full spec (with companion files) by default, and `--minimal` to opt out
- As a developer, I want to pass a natural-language feature description instead of a bare module name (e.g. `/specsync:create-spec "I want a feature that lets users export their data as CSV"`) and have the agent pick a module name and draft the spec's Purpose/Requirements from it
- As a developer, I want `specsync agents install` with no target flags to install all four tools at once
- As a developer, I want `specsync agents status` to show which tools are installed
- As a developer, I want `specsync agents uninstall` to cleanly remove only spec-sync's own files, never touching unrelated commands a tool's shared directory might contain

## Acceptance Criteria

- Four tools supported: Claude, Cursor, Codex, Gemini
- Claude, Cursor, and Gemini each get a skill plus `create-spec` and `create-change` commands; Codex gets a skill only
- Installation is idempotent per-artifact — re-installing an already-installed artifact is a no-op
- Uninstall never removes a tool's shared `commands/` directory, only spec-sync's own namespaced subdirectory/file within it
- Empty targets list means "all tools"
- `cmd_install` exits with code 1 if any tool installation fails

## Constraints

- Must never write outside the project root (no `~/.codex/prompts/`-style global writes), even where other ecosystem tools (e.g. OpenSpec) do
- No `toml` crate dependency exists in this project — Gemini's TOML command file must be a hand-built string template
- Must not assume undocumented behavior of a tool's command/skill format — verify against the tool's actual documented mechanics before implementing

## Out of Scope

- Additional slash commands beyond `create-spec` and `create-change` (for example native `check`/`coverage`/`score` commands)
- Extending native skill/command installation to Copilot or the generic `AGENTS.md` fallback — those remain served by `hooks.rs`'s prose-instruction mechanism
- Persisting which tools were selected in config so a bare `specsync agents install` remembers prior choices

### REQ-agents-001

The `agents` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-agents-002

Generated agent integrations SHALL preserve complete user intent across each tool's documented argument syntax.

Acceptance Criteria

- Create-spec guidance removes supported flags before classifying the complete remaining input.
- A complete single module identifier is preserved unchanged.
- Quoted or unquoted natural-language descriptions are classified before a kebab-case module name is derived.
- Gemini create-change guidance uses `{{args}}` and contains no `$ARGUMENTS` reference.
- Every generated skill and create-change command quotes a free-text interview answer as one positional argument.
- Reinstalling all four integrations remains deterministic and idempotent.

### REQ-agents-003

Checked-in project-native create-spec commands SHALL remain exact current installer outputs and SHALL
classify the complete non-flag input before selecting or deriving a module name.

Acceptance Criteria

- Claude, Cursor, and Gemini checked-in commands remove standalone `--minimal` flags before
  classifying the remaining input.
- A bare identifier is preserved unchanged, while quoted or unquoted free text derives a meaningful
  kebab-case slug and never uses only its first word.
- Guidance demonstrates `--minimal` before and after both a bare module and a free-text description.
- Tests byte-compare freshly installed commands with checked-in assets and prove a second install is
  idempotent.

### REQ-agents-004

Generated agent artifacts SHALL be tracked by a versioned digest manifest so upgrades preserve
customized files and report conflicts.

Acceptance Criteria

- Installation records artifact path, tool, template version, and digest in a project-local manifest.
- Unchanged generated artifacts update idempotently.
- Customized artifacts are never overwritten or deleted and produce an actionable conflict.
- Uninstall removes only digest-matching managed artifacts and preserves shared directories.
- Legacy installations are adopted only when their bytes match a known generated template.

### REQ-agents-005

The agent artifact manifest SHALL be read as committed evidence rather than as a regenerable cache: a manifest carrying a field this SpecSync does not recognise SHALL still be used, and a manifest missing a field this SpecSync requires SHALL still be refused.

Acceptance Criteria
- A manifest written by a newer SpecSync of the same major version does not stop `agents install` or `init`, because the file is committed and shared and one contributor's upgrade must not brick the command for everyone else.
- A manifest record missing a field this SpecSync requires is still refused, so tolerance of unknown fields cannot be mistaken for accepting any shape.
- The manifest is not discarded on a parse failure, because it records the digest of exactly the bytes SpecSync last generated and is the only thing distinguishing an untouched artifact from an edited one.

### REQ-agents-check-audit-commands-001

`specsync agents install` SHALL generate `/specsync:check` and `/specsync:audit` command files for tools that support project-local commands, and skill prose SHALL teach the two-verb lifecycle model.

Acceptance Criteria
- Claude, Cursor, and Gemini receive check and audit command files.
- Skill content distinguishes `change check` (scoped spec↔code sync) from `change audit` (actives + living specs).
- Template version advances so upgrades refresh generated artifacts.
- Generated `change check` skill and command files describe spec↔code sync, not project test commands.
- Skill prose tells agents to clear context only when the `Handoff:` line says `safe`, and otherwise
  to do what `Before clearing:` names first; the template version advances so installed skills refresh.

### REQ-agents-006

Generated SDD skill text SHALL describe scoped review as recordable by the same actor who approved the definition. It SHALL NOT instruct agents to invent a second human identity for solo work.

Acceptance Criteria

- The generated skill's lifecycle steps tell the agent to record `change review` with the human who signed off, including when that human also recorded definition approval.
- The skill does not require picking a second identity solely to satisfy SpecSync.

