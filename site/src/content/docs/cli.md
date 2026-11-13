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
specsync check --spec auth               # one module (repeatable)
specsync check auth                     # same filter, positional SPEC
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
specsync mcp lock                       # print MCP tools lock JSON from live catalog
specsync mcp lock --write               # write `.specsync/mcp-tools.lock.json` (explicit only)
specsync mcp diff                       # fail-closed compare live catalog vs committed lock
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

Supports Claude Code (`CLAUDE.md`), Cursor (`.cursorrules`), GitHub Copilot (`.github/copilot-instructions.md`), `AGENTS.md`, and pre-commit hooks.

To refresh an existing pre-commit hook after upgrading the generator, run `specsync hooks uninstall --precommit` followed by `specsync hooks install --precommit` from the same project root. Use the same `--root <project>` on both commands for a nested project. Installing alone preserves an existing managed block; uninstalling removes only the selected project's managed block and preserves other hook content. See [Pre-commit hook](workflow.md#pre-commit-hook) for the quoting-fix upgrade guidance.

### `agents`

Install native SDD skills and supported slash commands for coding agents. This is separate from repository instruction snippets and Git hooks managed by `hooks`.

```bash
specsync agents install                    # install all four supported integrations
specsync agents install --claude --gemini # install selected integrations
specsync agents status                     # show installation status
specsync agents uninstall                  # remove generated integrations
```

Supports Claude Code, Cursor, Codex, and Gemini CLI. Claude and Gemini receive slash commands
such as `/specsync:create-spec`, `/specsync:create-change`, `/specsync:check`, and
`/specsync:audit`. Cursor uses flat names: `/specsync-create-spec`, `/specsync-create-change`,
`/specsync-check`, and `/specsync-audit`. Codex is skill-only. The installed skills use the
deterministic SpecSync 6.0 lifecycle (`change check` for one change; `change audit` for active
workspaces and living specs) and never grant human approvals on an agent's behalf.

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
specsync import github --all-issues --label spec  # import open issues with a specific label
specsync import jira PROJ-123              # import from Jira ticket
specsync import --from-dir ./docs/specs    # import from local directory
```

`--from-dir` preserves what the source declares: frontmatter (`module`, `version`, `status`,
`files`) and existing sections are kept, and only missing sections are scaffolded, so a
frontmatter-complete source passes `specsync check` immediately. A parseable source with no
`files:` frontmatter list imports as a draft with a warning that `check --strict` will fail
until `files` is filled in. GitHub and Jira imports still write
`status: draft` skeletons with `files: []` and empty Public API tables; the command warns that
`specsync check --strict` fails until you add the `files:` mapping and complete the sections
([#416](https://github.com/CorvidLabs/spec-sync/issues/416)).

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

### `doctor`

Report every problem in the change records, each with its fix, and apply the safe ones on request.

```bash
specsync doctor                            # list findings; writes nothing
specsync doctor --fix --dry-run            # show what --fix would do; writes nothing
specsync doctor --fix                      # apply the safe fixes
specsync doctor --fix --close-late <id> --actor <name> --reason <text>   # close a merged change late
specsync doctor --fix --close-late all --actor <name> --reason <text>    # every eligible change
```

Findings: a change merged but never archived, an accepted change whose inputs drifted (including an
`accepted` package left in `.specsync/archive/changes/`), a finalize that stopped halfway, stale
verification, and records another SpecSync version wrote. A check that cannot run is a finding too.

- **Safe** fixes, applied by `--fix`: resume a stopped finalize (workflow v1: archive an accepted
  change whose evidence is current) and the `migrate 5.0` ledger backfill.
- **Decisions**, applied only with `--close-late`, `--actor` and `--reason`: a late close moves a
  package that is on the remote default branch, byte for byte, into
  `.specsync/archive/closed-late/<date>-<id>/` with a `late-close.json` that says it was never
  re-checked and is not verified. Several IDs are all or nothing.
- **Report only**: everything else, including stale verification (`change check <id> --commit`).

SpecSync 6.0 to 6.2 never read `.specsync/archive/closed-late/`, so they neither trip on a
closed-late change nor count it as checked.

### `change`

Manage the single verified change workflow. Every command supports global `--json` output for agent clients.

```bash
specsync change new "Add passkeys" --kind feature --spec auth --path src/auth.rs
specsync change answer add-passkeys acceptance_criteria "Passkey login works"
# answer remaining questions and complete the selected artifacts before approval
specsync change list
specsync change show add-passkeys
specsync change status add-passkeys          # --timeout <secs> bounds a read; exit 124 names how to resume
specsync change approve add-passkeys --actor "Ada"
# implement and test the approved scope
specsync change check add-passkeys --commit
specsync change audit             # add --archives to also re-walk every archive's history (slow)
# run the local verify lane, then complete human review (archive-first CI waits for the archive tip)
specsync change review add-passkeys --reviewer "Ada"
specsync change finalize add-passkeys
# finalize commits the archive; push it and wait for checks before GitHub merges
```

`change answer` checks an answer before it stores anything. An unknown question id is refused
with the accepted ids. An empty answer is refused, and so is a list answer that names nothing:
`,` for `affected_specs`, `affected_paths` or `dependencies`, `[]` for `acceptance_criteria` or
`affected_paths`. Answer exactly `none` or `'[]'` to clear `affected_specs` or `dependencies`
(`'[]'` always clears; `none` is refused as ambiguous when a change or module is itself called
`none`; quote `'[]'`, because zsh reads a bare `[]` as a pattern). `public_contract` and
`architecture_risk` take `yes` or `no`. `no_spec_change` takes only `no`: it declares that a
change created with `--no-spec-change` changes specs after all, and drops its rationale. Like
every answer it needs a draft, so send an approved change back first; the next approval then
needs a delta for each affected spec. Every `affected_specs` module needs its spec at its
canonical path, with the module's directory and file spelled exactly and no symlink on the way,
unless the project has no specs yet. A new module's spec must exist before the module is
declared, including a module that so far exists only under `[modules]` in the config. An `--actor` is trimmed
and refused when it holds a newline, a control character or an invisible character other than
ZWNJ and ZWJ.

For a separately created draft successor called `update-ui`, dependency and exact succession
are declared before its scope approval. Set `PREDECESSOR_ENTRY_DIGEST` to the full
`specsync.acceptance-entry.v1` digest of the accepted predecessor entry; `--spec` names its
canonical owner. A digest or module placeholder is not valid evidence.

```bash
specsync change depend update-ui add-passkeys
specsync change supersede update-ui add-passkeys \
  --path src/auth.rs --spec auth --digest "$PREDECESSOR_ENTRY_DIGEST"
specsync change adopt --dry-run
```

`send-back` returns an approved, implementing or verifying change to draft so its scope can be
answered again. It needs `--actor` and `--reason`, keeps every earlier approval, and cancels the
definition approval, so the change must be approved again. Check refuses while an edit the
change applied before the send-back, and no longer declares, is still in the spec, and says what
to restore (the pre-change text of a modified or removed block, or taking out an added one):

```bash
specsync change send-back add-passkeys --actor "Ada" --reason "The session code moved to src/session.rs"
# answer the scope questions again, then approve and check
```

`reopen` supports eligible accepted or archived workflow-v2 evidence as well as legacy records:

```bash
specsync change reopen add-passkeys --actor "Ada" --reason "Review fixes changed governed inputs"
# resume through check --commit, human review, review, and finalize
```

Historical workflow-v1 correction examples retain numeric IDs:

```bash
specsync change correct CHG-0001-add-passkeys architecture_risk yes --actor "Ada" --reason "Review found architectural impact"
specsync change correct-owner CHG-0001-add-passkeys --path src/auth.rs --spec auth --actor "Ada" --reason "Owner omitted from the accepted affected-spec list"
specsync change correct-owner CHG-0001-add-passkeys --path src/a.rs --path src/b.rs --spec auth --actor "Ada" --reason "Batch repair"
specsync change correct-owner CHG-0001-add-passkeys --manifest owners.json --actor "Ada" --reason "Batch repair"
specsync change correct-owner CHG-0001-add-passkeys --all-missing --spec auth --actor "Ada" --reason "Assign every omitted owner"
specsync change archive CHG-0001-add-passkeys
```

`acceptance_criteria` preserves scalar prose exactly; use a JSON array of strings to provide multiple criteria. `affected_specs` and `affected_paths` retain comma- and newline-separated list input.

New changes require one digest-bound human scope approval. `change status` always names one next
action and ends with one `Handoff:` line (`safe`, `conditional`, or `not yet`, the reason, and the
steps to take before clearing context; `--json` carries it as `summary.handoff` — see
[Clearing context between steps](workflow.md#clearing-context-between-steps)); `change check` applies approved deltas and records targeted evidence for **one change**
(not archive history); `change audit` checks active workspaces and living specs; scoped human
review binds the implementation commit; and `change finalize` creates the dated archive in the same
PR without merging externally. Global `--strict`, project policy, and release/security
classification add validators to this same path. Existing `start`, `verify`, `accept`, `archive`,
`correct`, and `correct-owner` commands remain compatible with historical two-approval
evidence. `reopen` also repairs eligible workflow-v2 terminal evidence. Recovery preserves the
already-canonical semantic delta rather than applying it again. `change adopt` enables
SDD for an existing project and can import active/canonical OpenSpec or Spec Kit artifacts.

`summary.approval_valid` is false whenever a gate that needs the approval would refuse it: the
approved scope changed, or a semantic delta body was edited after the approval that recorded it.
`summary.changed_approved_deltas` lists the edited modules and is omitted when empty. In both
cases JSON and text `Next:` name `change approve`, the handoff is `not yet`, and `change audit`
reports the refusal `change check` gives. Editing `plan.md` or another execution artifact after
approval stales verification, not approval.

#### Read timeouts

`change status`, `change show`, `change list` and `change ship-status` accept `--timeout <secs>`. Without it they read `SPECSYNC_TIMEOUT`, and with neither there is no limit. Long phases check the deadline between items, such as each commit of the archive-introduction index or each archive's workflow history. Once it has passed, the command:

- writes nothing to any change,
- keeps every verdict it completed in the read cache under the Git directory, so a rerun continues from there,
- prints the phase, its progress and the exact command that resumes the work,
- exits **124**.

A read that never reaches a long phase finishes and prints its normal output, even with `--timeout 0`.

```text
specsync: change ship-status timed out after 30s during archive-introduction index (140/214). Nothing was written. Completed work is cached, so rerunning continues from there:
  specsync change ship-status add-passkeys
```

With `--json` the report goes to stdout:

```json
{"error":"timeout","command":"change status","timeout_secs":30,"phase":"archive-introduction index","progress":"140/214","wrote":false,"resume":"specsync change status add-passkeys --json"}
```

The resume command runs exactly as printed: every argument that holds anything but letters, digits and `_@%+=:,./-` is single-quoted, so a path with `$`, `;`, `*` or an apostrophe can't change what it runs. When the deadline came from `SPECSYNC_TIMEOUT`, the resume command begins with `SPECSYNC_TIMEOUT= `, so running it in the same shell doesn't time out again.

A tool that bounds SpecSync with its own timer can pass that limit as `--timeout`, and it gets the phase instead of a bare timeout.

`change status <id> --json` and `change show <id> --json` also carry a top-level
`approved_deltas` array, one entry per affected module, sorted by module. `change answer`,
`depend` and `supersede --json` carry the same key, built in the same transaction as their
`summary`. `change new --json` carries it too, always `[]`, because a new change is a draft. The key is always present and is `[]` outside the approved, implementing and verifying
states, and when the approved scope is stale. Each entry has `module` and `delta_intact`:

- `delta_intact` is `true` when the delta body matches the body the approval recorded (line
  endings aside), `false` when it was edited after approval, for the same reason `change check`
  refuses it, and `null` when the approval predates the delta binding, which means unknown, not
  tampered.
- Only when `delta_intact` is `true`, `requirements` and `sections` list the `added`, `modified`
  and `removed` keys of the body's `### REQUIREMENT` and `### SPEC SECTION` headings, and
  `materialized` says whether the canonical spec carries that body. `materialized` is `false`
  again after a re-approved correction until the next `change check`.
- When `delta_intact` is `true` but the current delta grammar refuses the body (an approval
  recorded under an older grammar), the entry carries `declarations_error` instead of those keys.

```json
"approved_deltas": [
  {
    "module": "shipping",
    "delta_intact": true,
    "requirements": { "added": ["REQ-shipping-002"], "modified": [], "removed": [] },
    "sections": { "added": [], "modified": ["Public API"], "removed": [] },
    "materialized": false
  }
]
```

A tool that links to a requirement not yet in `requirements.md` should accept it only while
`summary.approval_valid` is true and the module's `delta_intact` is `true`, and should label it
pending until `materialized` is `true`.

Actor and reviewer labels, relayed decisions and stored provider declarations do not authenticate
identity. Local finalization checks recorded content and history; authenticated provenance
requires separately configured hosted checks and policy verification. The scope approver may
perform the implementation review.

`change approve` and `change review` take an optional relayed decision for a decision a person
made elsewhere: `--words "<their exact words>" --where "<where and when>" --recorded-by "<agent>"`,
all three together. `change status` shows each approval and review as a recorded claim, with any
relayed decision on the next line; `--json` carries them under `recorded_claims`.

`change repair-relays <id>` repairs a damaged relayed decision in an active change, committed or not. Each damaged `relayed_by` in `approvals.json`, `review-attempts.json` or `review.json` is put back from the same record at `HEAD`, then from its twin, and with no copy only that `relayed_by` is removed. If `HEAD` lists the file but Git cannot read it, or `HEAD` does not resolve, the command refuses and writes nothing, naming `git fsck` and the rerun. Every ledger must read before anything is written, so a ledger damaged in some other way is refused with its own restore. Recovery commands quote the change id after `--`, for example `specsync change repair-relays -- 'add-passkeys'`. It prints one line per repair and writes nothing when nothing is damaged; `--json` prints `{"change", "repairs"}`. An archived change is refused, naming `git checkout HEAD -- <package>`.

```text
$ specsync change repair-relays add-passkeys
✓ Repaired relayed decisions in add-passkeys:
  review.json: put back from review-attempts.json attempt 1
  Every approval and review attempt is kept, verdicts included; no digest or gate reads a relayed decision, so nothing went stale.
```

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

Create a default `.specsync/config.toml` in the current directory, plus `.specsync/sdd.json` with the change workflow off (`enabled: false`) and the change/archive directories. It starts no interview; enable the workflow later with `specsync change adopt`.

```bash
specsync init
specsync init --repair                     # restore missing .specsync support files without touching config
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
| `--spec NAME` | Repeatable module filter on `check`. Same matching as the positional SPEC argument. `change check` records this form as `specsync check --spec <name>`. |
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
  "errors": ["specs/auth/auth.spec.md: Spec documents 'oldFunction' but no matching export found in source"],
  "manifest_notices": [],
  "notices": [],
  "passed": false,
  "skipped_links": [],
  "specs_checked": 12,
  "stale": [],
  "suppressed_warnings": [],
  "warnings": [
    "specs/auth/auth.spec.md: 1/2 exports documented",
    "specs/auth/auth.spec.md: Undocumented export 'newHelper' from src/auth.ts"
  ]
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


### Explicit historical receipt reconciliation

SpecSync 6.2.0 adds `specsync change reconcile-history ID --actor ACTOR --reason REASON` for an unaccepted workflow-v2 successor that already declares exact `supersede` obligations. This is a scope request, not approval or archival. It binds the existing definition base as an immutable receipt anchor and invalidates prior scope approval. Reapprove the definition, run fresh verification, record a fresh scoped review, then finalize normally.

The resulting schema-2 evidence explicitly marks historical file transitions **unproven**. It authenticates the complete original accepted package and frozen manifest at the anchor, preserves those bytes, and still verifies the current successor inputs at its actual acceptance commit. It does not claim retrospective product acceptance or historical before/after proof. Normal schema-1 succession retains its actual base-file check. Protected tool/policy inputs, symlinks and gitlinks are excluded from this bounded migration mode. Older releases must not be used to operate or qualify this new evidence mode.

Original receipt bytes remain anchored in Git after native archival. The authenticated accepted-state preimage and exact generated archived change markdown are the only metadata projections normalized for witness comparison; altered markdown and all other changed original package files remain refused.

## Obsolete reopened intent

A qualified recovery reader exposes:

    specsync change retire <id> --actor <agent> --reason <reason> --replacement <id> --authority-source <source>

The replacement must be independently accepted, fresh and retain complete signed path/owner
custody. Retirement does not pass the old criterion, satisfy dependencies or provide ordinary
semantic succession. Original scope, failures and receipts remain intact. Actor/source fields
are recorded claims, not authenticated human identity.

If the package move fails, durable state and receipt remain together and the command reports
a pending move. Identical retry validates custody and finishes it without a second disposition.
An eligible ordinary child may use send-back, answer dependencies with the accepted replacement,
then obtain fresh scope/check/review. A frozen dependent cannot be silently rebound.

Default audit reports that archived retirement custody was not inspected. Use change audit
--archives for archive-history/custody validation. Explicit retired status and finalization
also refuse invalid custody when relying on it.

Published 6.2 readers do not understand the new retired state. Every required lifecycle
consumer needs an explicitly qualified updated reader. Structural checks or existing site Trust
do not establish this compatibility. This candidate is unreleased; no merge/release/global install.


### Explicit schema-v2 retirement custody

New receipts use specsync.retirement.v2. Existing v1 receipts retain their original digest
preimage and accepted-source/equal-owner semantics; they are not rewritten.
The default source is accepted-receipt. Only when no Accepted-state or mismatch-anchor
history exists may an operator explicitly request --source-kind
committed-audited-reopening-unproven. It authenticates the complete pre-existing committed
audited-reopening package, ancestor and directory, frozen definition, ledger, prior
verification/manifest and retained closing/finalization/review bytes. It proves retained
claims, never a standalone Accepted snapshot or human provenance. Missing, changed,
nonancestor or unavailable history remains blocked.

Both authenticated owner vectors remain in custody. The exceptional
unchanged-exact-file-current-owner classification requires old owners exactly
[@exact:delivery], ordinary File on both sides, identical path/kind/mode/payload/entry
digests and nonempty named replacement owners. It supplies no ordinary succession,
dependency or accepted implementation credit. Missing, symlink, directory and named-owner
remapping cannot use this exception. All other complete-custody and current replacement
authentication checks still apply.
