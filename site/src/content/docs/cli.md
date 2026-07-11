---
title: "CLI Reference"
section: "Reference"
order: 0
---

---

## Usage

```
specsync [command] [flags]
```

Default command is `check`.

---

## Commands

### `check`

Validate all specs against source code.

```bash
specsync check                          # basic validation
specsync check --strict                 # warnings become errors
specsync check --strict --require-coverage 100
specsync check --json                   # machine-readable output
```

Three validation stages:
1. **Structural** — required frontmatter fields, file existence, required sections
2. **API surface** — spec symbols vs. actual code exports
3. **Dependencies** — `depends_on` paths, `db_tables` against schema

### `coverage`

File and module coverage report.

```bash
specsync coverage
specsync coverage --json
```

### `generate`

Scaffold spec files for modules that don't have one. Uses `specs/_template.spec.md` if present.

```bash
specsync generate                          # template mode — guided starter specs
specsync generate --provider anthropic     # AI mode — use the Anthropic API
specsync generate --provider openai --model gpt-4o   # AI mode — pick provider + model
specsync generate --provider ollama        # AI mode — keyless local Ollama
```

With a provider configured, source code is sent to an LLM which generates filled-in specs (Purpose, Public API tables, Invariants, etc.). AI calls go through the shared [`corvid-ai`](https://crates.io/crates/corvid-ai) crate over plain HTTP — no CLI tool required. Just set `<PROVIDER>_API_KEY` and name the provider:

| Provider | How it works |
|:---------|:-------------|
| `anthropic` | Anthropic Messages API (`ANTHROPIC_API_KEY`). Default model `claude-sonnet-4-6` |
| `openai` | OpenAI Chat Completions API (`OPENAI_API_KEY`). Requires an explicit `--model` |
| `openrouter` | OpenRouter API (`OPENROUTER_API_KEY`) |
| `gemini` | Google Gemini API (`GEMINI_API_KEY`) |
| `deepseek` | DeepSeek API (`DEEPSEEK_API_KEY`) |
| `groq` | Groq API (`GROQ_API_KEY`) |
| `mistral` | Mistral API (`MISTRAL_API_KEY`) |
| `xai` | xAI API (`XAI_API_KEY`) |
| `together` | Together API (`TOGETHER_API_KEY`). Requires an explicit `--model` |
| `ollama` | Local Ollama HTTP API (`http://localhost:11434`, keyless) or Ollama Cloud (`OLLAMA_API_KEY`). Default model `llama3.3` |
| `claude` | **Deprecated** — warns and routes to the `anthropic` API |
| `copilot`, `cursor` | **Deprecated** |

With no provider configured, `generate` auto-detects: no key anywhere falls back to keyless local Ollama; exactly one `<PROVIDER>_API_KEY` selects that provider; multiple keys prompt an interactive picker (TTY) or use a deterministic order. Add `--model <id>` to override the provider default.

See [Configuration](configuration.md) for `aiProvider`, `aiModel`, `aiCommand`, `aiApiKey`, `aiBaseUrl`, and `aiTimeout`.

### `score`

Quality-score your spec files on a 0–100 scale with per-spec improvement suggestions.

```bash
specsync score                          # score all specs
specsync score --json                   # machine-readable scores
```

Scores are based on a weighted rubric: completeness, detail level, API table coverage, behavioral examples, and more.

### `mcp`

Start SpecSync as an MCP (Model Context Protocol) server over stdio. Enables AI agents like Claude Code, Cursor, and Windsurf to use SpecSync tools natively.

```bash
specsync mcp                            # start MCP server (stdio JSON-RPC)
```

Exposes tools: `specsync_check`, `specsync_generate`, `specsync_coverage`, `specsync_score`.

### `add-spec`

Scaffold a single spec with companion files (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md` if enabled).

```bash
specsync add-spec auth                     # creates specs/auth/auth.spec.md + companions
```

Companion files sit alongside the spec and give agents structured context:
- **`requirements.md`** — user stories, acceptance criteria, constraints (authored by Product/Design)
- **`tasks.md`** — outstanding work items for the module
- **`context.md`** — design decisions, constraints, history

### `init-registry`

Generate a `.specsync/registry.toml` listing all modules in the project. Other projects reference your modules via this registry.

```bash
specsync init-registry                     # uses project folder name
specsync init-registry --name myapp        # custom registry name
```

Commit the generated file to your repo's default branch so `resolve --remote` can find it.

### `resolve`

Verify that all `depends_on` references in your specs actually exist. By default checks local paths only (no network).

```bash
specsync resolve                           # verify local refs
specsync resolve --remote                  # also verify cross-project refs via GitHub
```

Cross-project refs use the `owner/repo@module` syntax in `depends_on`. The `--remote` flag fetches the target repo's `.specsync/registry.toml` from GitHub to confirm the module exists. See [Cross-Project References](cross-project-refs.md) for details.

### `hooks`

Install agent instruction files and git hooks so AI agents and contributors stay spec-aware.

```bash
specsync hooks install                     # install agent instructions + pre-commit hook
specsync hooks uninstall                   # remove installed hooks
specsync hooks status                      # check what's installed
```

Supports Claude Code (`CLAUDE.md`), Cursor (`.cursor/rules`), GitHub Copilot (`.github/copilot-instructions.md`), and pre-commit hooks.

### `compact`

Trim older changelog entries from specs to prevent unbounded growth.

```bash
specsync compact --keep 10              # keep last 10 entries per spec
specsync compact --keep 5 --dry-run     # preview what would be removed
```

### `archive-tasks`

Archive completed tasks from companion `tasks.md` files.

```bash
specsync archive-tasks                  # move completed tasks to archive section
specsync archive-tasks --dry-run        # preview what would be archived
```

### `view`

View specs filtered by role — shows only the sections relevant to a specific audience.

```bash
specsync view --role dev                # developer view
specsync view --role qa                 # QA view
specsync view --role product            # product manager view
specsync view --role agent              # AI agent view
specsync view --role dev --spec auth    # specific spec, developer view
```

### `new`

Quick-create a minimal spec with auto-detected source files. Faster than `add-spec` when you just need the spec.

```bash
specsync new auth                          # creates specs/auth/auth.spec.md
specsync new auth --full                   # also creates companion files (requirements.md, tasks.md, context.md, testing.md, and design.md if enabled)
```

Scans `sourceDirs` for files matching the module name to auto-populate the `files:` frontmatter field.

### `migrate`

Upgrade a 3.x project to the v4.0.0 layout. Moves config into `.specsync/`, converts to TOML, extracts lifecycle history, and stamps the version.

```bash
specsync migrate                           # run full migration
specsync migrate --dry-run                 # preview what would change
specsync migrate --no-backup               # skip backup creation
specsync migrate --json                    # machine-readable output
```

The migration is step-based and idempotent — re-running on a partially migrated project resumes from where it left off. A backup is created in `.specsync/backup-3x/` before any destructive changes.

### `rehash`

Regenerate the hash cache for all specs. Useful after `git pull`, branch switches, or manual spec edits to reset the incremental validation baseline.

```bash
specsync rehash                            # rebuild .specsync/hashes.json
```

> **Note:** The hash cache (`.specsync/hashes.json`) should **not** be committed to git — it is a local-only optimization for incremental validation. Both `specsync init` and `specsync migrate` automatically add it to `.gitignore`. In CI, use `specsync check --force` (the GitHub Action does this by default).

### `stale`

Identify specs that haven't been updated since their source files changed. Uses git history to compare the last spec commit against source file commits.

```bash
specsync stale                             # show all stale specs
specsync stale --threshold 5              # only flag specs 5+ commits behind
specsync stale --json                      # machine-readable output
```

### `report`

Per-module coverage report with stale and incomplete detection. Combines coverage, staleness, and validation into a single dashboard.

```bash
specsync report                            # full module health report
specsync report --json                     # machine-readable output
specsync report --stale-threshold 5       # custom staleness threshold
```

### `comment`

Post spec-sync check results as a PR comment. Useful in CI to surface spec drift directly in pull requests.

```bash
specsync comment --pr 42                   # post comment to PR #42
specsync comment --pr 42 --base main       # compare against specific base branch
specsync comment                           # print comment body to stdout (no posting)
```

Requires `GITHUB_TOKEN` environment variable when posting. The comment includes a markdown diff of exports added/removed. Existing SpecSync comments are updated rather than duplicated.

### `deps`

Validate the cross-module dependency graph. Detects cycles, missing dependencies, and undeclared imports.

```bash
specsync deps                              # validate dependency graph
specsync deps --json                       # machine-readable output
specsync deps --mermaid                    # output Mermaid diagram
specsync deps --dot                        # output Graphviz DOT
```

### `scaffold`

Scaffold a spec with optional directory and template overrides.

```bash
specsync scaffold auth                     # scaffold in default specs dir
specsync scaffold auth --dir modules       # scaffold in custom directory
specsync scaffold auth --template custom   # use custom template
```

### `import`

Import specs from external sources — GitHub Issues, Jira, or local directories.

```bash
specsync import github 123                 # import from GitHub issue #123
specsync import github --all-issues        # import all open issues as specs
specsync import github --label spec        # import issues with specific label
specsync import jira PROJ-123              # import from Jira ticket
specsync import --from-dir ./docs/specs    # import from local directory
```

### `wizard`

Interactive guided spec creation. Prompts for module name, source files, dependencies, and completes sections interactively.

```bash
specsync wizard
```

### `issues`

Verify that GitHub issue references in spec frontmatter point to real issues. Optionally create missing issues.

```bash
specsync issues                            # verify issue references
specsync issues --create                   # create GitHub issues for specs with errors
specsync issues --json                     # machine-readable output
```

### `changelog`

Generate a changelog of spec changes between two git refs.

```bash
specsync changelog v3.3.0..v3.4.0         # changes between tags
specsync changelog HEAD~10..HEAD           # recent changes
specsync changelog v3.3.0..v3.4.0 --json  # machine-readable output
```

### `merge`

Auto-resolve git merge conflicts in spec files. Understands spec structure to make intelligent merge decisions.

```bash
specsync merge                             # resolve conflicts in conflicted specs
specsync merge --dry-run                   # preview resolutions without writing
specsync merge --all                       # process all conflicted files
```

### `rules`

Display configured validation rules and their current status (built-in rules, custom rules, severity levels).

```bash
specsync rules                             # show all rules and their configuration
```

### `change`

Manage the complete verified SDD delivery lifecycle. Every command supports global `--json` output for agent clients.

```bash
specsync change new "Add passkeys" --kind feature --spec auth --path src/auth.rs
specsync change answer CHG-0001-add-passkeys acceptance_criteria "Passkey login works"
specsync change depend CHG-0002-update-ui CHG-0001-add-passkeys
specsync change list
specsync change show CHG-0001-add-passkeys
specsync change approve CHG-0001-add-passkeys
specsync change start CHG-0001-add-passkeys
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys
specsync change archive CHG-0001-add-passkeys
specsync change check
specsync change adopt --dry-run
```

Definition and closing approvals are mandatory and digest-bound. `change adopt` enables SDD for an existing project and can import active/canonical OpenSpec or Spec Kit artifacts.

### `lifecycle`

Manage spec status transitions. Supports `promote`, `demote`, `set`, `status`, `history`, `guard`, `auto-promote`, and `enforce` subcommands.

```bash
specsync lifecycle status                  # show status of all specs
specsync lifecycle status auth             # show status of a specific spec
specsync lifecycle promote auth            # advance: draft → review → active → stable
specsync lifecycle demote auth             # step back one status level
specsync lifecycle set auth deprecated     # jump to any status
specsync lifecycle set auth review --force # skip transition validation
specsync lifecycle history auth            # view transition audit log
specsync lifecycle guard auth              # dry-run: check all valid transitions
specsync lifecycle guard auth active       # dry-run: check specific transition
specsync lifecycle auto-promote            # promote all specs that pass guards
specsync lifecycle auto-promote --dry-run  # preview what would be promoted
specsync lifecycle enforce --all           # CI mode: check all lifecycle rules
specsync lifecycle enforce --require-status # require all specs to have a status field
specsync lifecycle enforce --max-age       # flag specs stuck too long in a status
specsync lifecycle enforce --allowed       # check specs are in allowed statuses
```

**Transition rules:**
- `promote` advances one step: draft → review → active → stable
- `demote` steps back one level
- `set` allows jumping to any status, with validation that the transition is sensible
- Any non-terminal status can jump directly to `deprecated`
- Use `--force` to override both transition validation and guards
- Supports `--format json` for machine-readable output

**Transition guards:**
- Configure in `.specsync/config.toml` under `[lifecycle.guards]` (see [Configuration](configuration.md))
- Guards can require minimum score, required sections, or no-stale status
- Use `lifecycle guard` to dry-run guard checks without changing status
- Blocked transitions show which guards failed and why

**Transition history:**
- When `lifecycle.trackHistory` is enabled (default), transitions are recorded in `.specsync/lifecycle/<module>.json`
- Use `lifecycle history <spec>` to view the full audit trail

**Auto-promote:**
- Scans all specs and promotes any whose next transition passes all configured guards
- History entries are tagged `(auto-promote)` for audit clarity
- Use `--dry-run` to preview without modifying files

**CI enforcement (`enforce`):**
- `--require-status`: every spec must have a valid `status` field in frontmatter
- `--max-age`: flag specs stuck in a status longer than configured in `[lifecycle] max_age` (days per status)
- `--allowed`: require all specs to have a status in `[lifecycle] allowed_statuses`
- `--all`: run all three checks at once
- Exits non-zero if any violations are found — designed for CI pipelines

### `diff`

Show API changes since a git ref.

```bash
specsync diff --base main               # changes since main branch
specsync diff --base HEAD~5             # changes in last 5 commits
specsync diff --base v1.0.0 --json     # machine-readable output
```

### `init`

Create a default `.specsync/config.toml` in the current directory.

```bash
specsync init
```

### `watch`

Live validation — re-runs on file changes with 500ms debounce. `Ctrl+C` to exit.

```bash
specsync watch
```

---

## Flags

| Flag | Description |
|:-----|:------------|
| `--strict` | Warnings become errors. Recommended for CI. |
| `--require-coverage N` | Fail if file coverage < N%. |
| `--root <path>` | Project root directory (default: cwd). |
| `--provider <name>` | Enable AI-powered generation and select provider: `anthropic`, `openai`, `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, or `ollama` (deprecated: `claude`→`anthropic`, `copilot`, `cursor`). Without this flag (and no AI config/env), `generate` uses templates only. |
| `--model <id>` | Override the provider's default model (e.g. `gpt-4o`, `claude-sonnet-4-6`). Resolution is `--model` > `SPECSYNC_AI_MODEL` > `aiModel` config > provider default. |
| `--format <fmt>` | Output format: `text` (default), `json`, or `markdown`. Markdown produces clean tables suitable for PRs and docs. |
| `--json` | Shorthand for `--format json`. Structured output, no color codes. |
| `--fix` | Auto-add undocumented exports as stub rows in spec Public API tables (on `check`). |
| `--force` | Skip hash cache and re-validate all specs (on `check`). Override transition validation (on `lifecycle`). |
| `--create-issues` | Create GitHub issues for specs with validation errors (on `check`). |
| `--dry-run` | Preview changes without writing files (on `compact`, `archive-tasks`, `merge`). |
| `--stale N` | Flag specs N+ commits behind their source files (on `check`). |
| `--exclude-status <s>` | Exclude specs with the given status from processing. Repeatable. |
| `--only-status <s>` | Only process specs with the given status. Repeatable. |
| `--mermaid` | Output dependency graph as Mermaid diagram (on `deps`). |
| `--dot` | Output dependency graph as Graphviz DOT (on `deps`). |
| `--full` | Include companion files when creating a spec (on `new`). |
| `--all` | Process all items, not just the first match (on `merge`, `score`). |

---

## Exit Codes

| Code | Meaning |
|:-----|:--------|
| `0` | All checks passed |
| `1` | Errors found, warnings with `--strict`, or coverage below threshold |

---

## JSON Output

### Check

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

### Coverage

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "loc_coverage": 79.12,
  "loc_covered": 4200,
  "loc_total": 5308,
  "modules": [{ "name": "helpers", "has_spec": false }],
  "uncovered_files": [{ "file": "src/helpers/utils.ts", "loc": 340 }]
}
```
