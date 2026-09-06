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
specsync generate                          # deterministic guided starter specs
```

Generation is local and deterministic. It never accepts provider or model flags, reads API keys, sends source to a model, or executes an AI command. Use `specsync agents install` or `specsync mcp` for enrichment through your coding agent's own trust boundary.

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
specsync mcp                            # read-only MCP server (stdio JSON-RPC)
specsync mcp --allow-write              # additionally expose root-confined init/generate tools
```

By default, the server exposes five read tools: `specsync_check`, `specsync_coverage`,
`specsync_list_specs`, `specsync_score`, and `specsync_issues`. Read-tool `root` overrides must name
the configured server root or an existing directory beneath it after canonicalization. Traversal,
nonexistent paths, outside paths, and symlink escapes are rejected.

The same boundary applies before and after project configuration loads: config/metadata/cache files,
`specs_dir`, `source_dirs`, `schema_dir`, manifest workspace paths, dependency references, module
files/names, spec file mappings, and nested symlink targets must remain inside the canonical server
root. Invalid paths fail before project data is read or generated. Confinement and source
autodetection scans are bounded and honor ignored or configured-excluded directories.

`--allow-write` is an explicit capability grant. It additionally exposes `specsync_generate` and
`specsync_init`; these mutating tools always operate at the configured server root and reject a
per-call `root` argument. Exact tool schemas reject unknown keys and wrong value types.

**Migration note:** MCP clients that expect `specsync_generate` or `specsync_init` in the default
tool list must add `--allow-write` to the server command. Grant it only when that client should be
able to modify the configured project.

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

### `agents`

Install native SDD skills and supported slash commands for coding agents. This is separate from repository instruction snippets and Git hooks managed by `hooks`.

```bash
specsync agents install                    # install all detected integrations
specsync agents install --claude --gemini # install selected integrations
specsync agents status                     # show installation status
specsync agents uninstall                  # remove generated integrations
```

Supports Claude Code, Cursor, Codex, and Gemini CLI. Claude, Cursor, and Gemini also receive slash commands for `create-spec`, `create-change`, **`check`**, and **`audit`** (`/specsync:check`, `/specsync:audit`). Codex is skill-only. The installed skills use the deterministic SpecSync 6.0 lifecycle (`change check` for one change; `change audit` for active workspaces and living specs) and never grant human approvals on an agent's behalf.

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

Scans `source_dirs` for files matching the module name to auto-populate the `files:` frontmatter field.

### `migrate`

Upgrade a legacy 3.x project to the current layout. Moves config into `.specsync/`, converts it to
TOML, extracts lifecycle history, and stamps the project version. Existing projects can then adopt
the current verified lifecycle with `specsync change adopt`; adoption sets `enabled: true` on a
policy written off, and otherwise existing policy and lifecycle evidence remain byte-identical while
subsequent changes use the 6.0 single workflow. Adoption fails before
writing when an uncommitted or branch-only legacy change is absent from the trusted comparison
cutoff, and publishes its report, imports, and baseline atomically.

```bash
specsync migrate                           # run full migration
specsync migrate --dry-run                 # preview what would change
specsync migrate --no-backup               # skip backup creation
specsync migrate --json                    # machine-readable output
```

The migration is step-based and idempotent — re-running on a partially migrated project resumes from where it left off. A backup is created in `.specsync/backup-3x/` before any destructive changes.

#### `migrate 5.0`

Backfill the 5.1 reopening digest fields (`stale_acceptance_input_digest` / `current_acceptance_input_digest`) on 5.0.1-era change ledgers. Use this when `specsync check` fails on a historical `approvals.json` with a missing-field error — the error message names this command as the remediation.

```bash
specsync migrate 5.0                       # repair reopening records across active and archived changes
specsync migrate 5.0 --dry-run             # report planned repairs without writing
specsync migrate 5.0 --json                # machine-readable report
```

The backfill is deterministic (`stale` reproduces the embedded prior-verification digest, `current` comes from the superseding verification or a live recomputation), idempotent (re-running changes nothing), and verification-gated (each repaired ledger must re-parse before the write lands). A reopening that cannot be repaired deterministically fails its change without mutating that ledger; other changes still migrate. Bare `specsync migrate` keeps the 3.x→4.0 pipeline and never touches change ledgers.

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

Manage the single verified change workflow. Every command supports global `--json` output for agent clients.

```bash
specsync change new "Add passkeys" --kind feature --spec auth --path src/auth.rs
specsync change answer CHG-0001-add-passkeys acceptance_criteria "Passkey login works"
specsync change depend CHG-0002-update-ui CHG-0001-add-passkeys
specsync change list
specsync change show CHG-0001-add-passkeys
specsync change status CHG-0001-add-passkeys
specsync change approve CHG-0001-add-passkeys
specsync change check CHG-0001-add-passkeys
specsync change audit
specsync change review CHG-0001-add-passkeys --reviewer "Independent reviewer"
specsync change finalize CHG-0001-add-passkeys
# GitHub branch protection or merge queue performs the merge
# Historical repair commands remain available:
specsync change reopen CHG-0001-add-passkeys --actor "Ada" --reason "Review fixes changed governed inputs"
specsync change correct CHG-0001-add-passkeys architecture_risk yes --actor "Ada" --reason "Review found architectural impact"
specsync change correct-owner CHG-0001-add-passkeys --path src/auth.rs --spec auth --actor "Ada" --reason "Owner omitted from the accepted affected-spec list"
specsync change correct-owner CHG-0001-add-passkeys --path src/a.rs --path src/b.rs --spec auth --actor "Ada" --reason "Batch repair"
specsync change correct-owner CHG-0001-add-passkeys --manifest owners.json --actor "Ada" --reason "Batch repair"
specsync change correct-owner CHG-0001-add-passkeys --all-missing --spec auth --actor "Ada" --reason "Assign every omitted owner"
specsync change supersede CHG-0002-update-ui CHG-0001-add-passkeys --path specs/auth/auth.spec.md --module auth
specsync change archive CHG-0001-add-passkeys
specsync change adopt --dry-run
```

`acceptance_criteria` preserves scalar prose exactly; use a JSON array of strings to provide multiple criteria. `affected_specs` and `affected_paths` retain comma- and newline-separated list input.

New changes require one digest-bound human scope approval. `change status` always names one next
action and ends with one `Handoff:` line (`safe`, `conditional`, or `not yet`, the reason, and the
steps to take before clearing context; `--json` carries it as `summary.handoff` — see
[Clearing context between steps](workflow.md#clearing-context-between-steps)); `change check` applies approved deltas and records targeted evidence for **one change**
(not archive history); `change audit` checks active workspaces and living specs; independent scoped
review binds the implementation commit; and `change finalize` creates the dated archive in the same
PR without merging externally. Global `--strict`, project policy, and release/security
classification add validators to this same path. Existing `start`, `verify`, `accept`, `archive`,
`reopen`, `correct`, and `correct-owner` commands remain compatible with historical two-approval
evidence. Neither repair path reapplies an already-canonical semantic delta. `change adopt` enables
SDD for an existing project and can import active/canonical OpenSpec or Spec Kit artifacts.

Use `change correct-owner` to append audited exact canonical owner corrections for reopened acceptance evidence — for example owners omitted from a historical affected-spec list. The single form takes one `--path`/`--spec` pair; the batch form accepts repeated `--path` flags (one shared `--spec` or paired lists), a `--manifest` file (JSON `[{path, module}]` or TSV), or `--all-missing --spec <module>` to discover every production-source affected path that lacks canonical ownership. Every entry validates independently against the single-correction rules, and the batch is transactional: if any entry is invalid, no corrections from the batch are persisted. Use `change supersede` before definition approval when a later change adopts an exact predecessor path/module obligation, so the predecessor's accepted evidence remains successor-covered instead of going stale. A predecessor entry signed under a reserved exact owner (`@exact:test`, `@exact:delivery`) can be adopted by any module that owns the path now through `[modules."<name>"] owns` in `.specsync/config.toml`; no reopen or owner correction is needed.

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
- When `lifecycle.track_history` is enabled (default), transitions are recorded in `.specsync/lifecycle/<module>.json`
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
| `--format <fmt>` | Output format: `text` (default), `json`, `markdown`, `github`, `table`, or `csv`. |
| `--json` | Shorthand for `--format json`. Structured output, no color codes. |
| `--fix` | Auto-add undocumented exports as stub rows in spec Public API tables (on `check`). |
| `--force` | Skip hash cache and re-validate all specs (on `check`). Override transition validation (on `lifecycle`). |
| `--create-issues` | Create GitHub issues for specs with validation errors (on `check`). |
| `--dry-run` | Preview supported write operations without changing files. Availability is shown in each command's help. |
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

`file_coverage` and `loc_coverage` are `null` when the denominator is zero —
no source files were discovered, or the discovered files hold no lines. A
percentage over nothing would be indistinguishable from full coverage, so no
percentage is reported and `--require-coverage` fails closed. `files_total`
counts the files discovered on disk plus any a spec's `files:` list names that
are absent, since an absent file can never be covered.
