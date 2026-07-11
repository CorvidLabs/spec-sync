---
title: "Workflow Guide"
section: "Reference"
order: 2
---

End-to-end walkthrough of the verified SpecSync 5.0 SDD workflow.

---

## The Change Lifecycle

Every delivery change goes through a predictable lifecycle:

```text
draft → approved → implementing → verifying → accepted → archived
```

| State | What happens | Key commands |
|:------|:-------------|:-------------|
| **Draft** | Deterministic interview selects scope and adaptive artifacts | `change new`, `change answer` |
| **Approved** | A human approves the definition digest | `change approve` |
| **Implementing** | Code follows canonical specs plus approved deltas | `change start`, `check` |
| **Verifying** | Tests and requirement evidence are recorded | `change verify` |
| **Accepted** | Closing approval atomically updates canonical truth | `change accept` |
| **Archived** | The immutable workspace moves to dated history | `change archive` |

Module maturity (`draft → review → active → stable → deprecated → archived`) remains separately available through `specsync lifecycle`.

## 1. Create and Interview

```bash
specsync change new "Add passkeys" --spec auth --path src/auth.rs --json
specsync change answer CHG-0001-add-passkeys acceptance_criteria \
  "A registered passkey authenticates the user" --json
specsync change answer CHG-0001-add-passkeys public_contract yes --json
specsync change answer CHG-0001-add-passkeys architecture_risk yes --json
```

The shared deterministic engine asks only unresolved questions and selects requirements, research, design, plan, tasks, context, testing, docs, or custom artifacts according to change type and risk. Agent skills present the same questions conversationally.

## 2. Approve and Implement

Complete selected artifacts and semantic deltas, then obtain explicit human approval:

```bash
specsync change approve CHG-0001-add-passkeys
specsync change start CHG-0001-add-passkeys
```

Requirements use stable IDs, normative SHALL statements, and acceptance criteria. Changing an approved artifact invalidates its digest and blocks progress until reapproved.

## 3. Verify, Accept, and Archive

```bash
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys
# merge the delivery branch before archiving
specsync change archive CHG-0001-add-passkeys
```

Verification runs only project-configured commands without a shell and streams their output. Its evidence is bound to both the commit and the tested working-tree inputs, so source, test, configuration, or contract edits require a fresh verification. Acceptance requires successful evidence, requirement-to-test/API traceability, complete tasks, conflict-free deltas, and closing human approval.

Archive after the delivery branch is merged (or otherwise no longer differs from its comparison base). Until then, SpecSync keeps the accepted workspace active because the delivery diff still depends on its path coverage. This prevents the common gap where accepting and immediately archiving makes an unmerged implementation look unspecced.

The repository includes executable examples for a [complete lifecycle](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-lifecycle), [ordered concurrent changes](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-concurrent-changes), and a [five-epic product evolution](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-five-epics). Each creates a disposable Git project and runs the real CLI end to end.

---

## Project Setup

### Initialize a project

```bash
specsync init
```

This creates `.specsync/config.toml`, `.specsync/sdd.json`, the change/archive directories, detects verification commands, and offers native agent integration plus a first change interview. Existing 4.x projects remain unchanged until `specsync change adopt`.

### Install hooks and agent instructions

```bash
specsync hooks install
```

This installs:
- **Agent instructions** — `CLAUDE.md`, `.cursor/rules`, `.github/copilot-instructions.md`, `AGENTS.md` — so AI coding tools know to respect specs
- **Pre-commit hook** — runs `specsync check` before every commit, blocking commits with spec errors

Check what's installed with `specsync hooks status`.

---

## Creating Canonical Specs

### Option A: Scaffold a single module

```bash
specsync add-spec auth
```

Creates `specs/auth/` with five files by default, plus optional `design.md` when design artifacts are enabled:

| File | Purpose | Who writes it |
|:-----|:--------|:--------------|
| `auth.spec.md` | Technical contract — frontmatter, Public API, Invariants | Developer / Architect |
| `requirements.md` | User stories, acceptance criteria, constraints | Product / Design |
| `tasks.md` | Outstanding work items, review sign-offs | Anyone |
| `context.md` | Design decisions, key files, current status | Developer / Agent |
| `testing.md` | Test strategy, QA checklists, edge cases | QA / Developer |
| `design.md` *(opt-in)* | Layout, component hierarchy, design tokens | Design / Frontend |

The spec file is the only one SpecSync validates against code. The companion files provide structured context for humans and AI agents working on the module.

> **Convention:** Requirements (user stories, acceptance criteria) belong in `requirements.md`, not as inline sections in the spec. Non-draft specs with inline `## Requirements` or `## Acceptance Criteria` sections will produce a warning.

### Option B: Scaffold all unspecced modules

```bash
specsync generate                       # deterministic guided starter specs
specsync agents install                 # install native agent workflow
```

Generation never sends source to a model. Your coding agent can enrich the scaffold through the installed skill or MCP, using its own permissions. Always review the result and run `specsync check` immediately after.

### Option C: Write specs by hand

Create `specs/<module>/<module>.spec.md` with the required frontmatter (`module`, `version`, `status`, `files`) and sections. See [Spec Format](spec-format.md) for the full reference.

---

## 3. Validating Specs

### Basic validation

```bash
specsync check
```

Three stages run in order:

1. **Structural** — required frontmatter fields, file existence, required sections
2. **API surface** — spec symbols vs. actual code exports (bidirectional)
3. **Dependencies** — `depends_on` paths, `db_tables` against schema

Errors mean the spec references something that doesn't exist in code. Warnings mean code exports something the spec doesn't document.

### Auto-fix undocumented exports

```bash
specsync check --fix
```

Adds review rows to your Public API tables for any undocumented exports. You still need to replace the generated description prompts, but the symbol names are correct.

### Strict mode (for CI)

```bash
specsync check --strict
specsync check --strict --require-coverage 100
```

`--strict` promotes warnings to errors — every export must be documented. `--require-coverage` fails if file coverage drops below the threshold.

---

## 4. Iterating Until Clean

The typical iteration loop:

```bash
specsync check                    # see what's wrong
# fix errors — rename symbols, add missing exports, update file paths
specsync check                    # verify fixes
# repeat until clean
```

Common fixes:

| Error | Fix |
|:------|:----|
| Phantom export `foo` not found in source | Remove `foo` from the spec, or add it to the code |
| Undocumented export `bar` | Add `bar` to the Public API table |
| File `src/old.ts` not found | Update the `files` list in frontmatter |
| Required section missing | Add the section heading and content |

When working with an AI agent, pipe `--json` output for structured error handling:

```bash
specsync check --json
# Agent reads JSON, fixes each error, re-runs check
```

---

## 5. Measuring Quality

### Coverage

```bash
specsync coverage
```

Shows file and LOC coverage — what percentage of your source code has a spec. Use `--json` to get machine-readable output with `uncovered_files` sorted by size, so you can prioritize the largest gaps.

### Quality score

```bash
specsync score
```

Scores each spec on a 0–100 scale based on completeness, detail, API table coverage, behavioral examples, and more. Each spec gets a letter grade and specific improvement suggestions.

---

## 6. Ongoing Maintenance

### Watch mode

```bash
specsync watch
```

Re-validates on every file change (500ms debounce). Useful during active development — you'll see spec drift the moment it happens.

### Diffing against a ref

```bash
specsync diff --base main
specsync diff --base HEAD~5
```

Shows API changes since a git ref — what was added, removed, or changed. Good for reviewing what spec updates a PR needs.

### Keeping specs in sync with code changes

When you rename, add, or remove exports:

1. Run `specsync check` to see what drifted
2. Update the spec's Public API table
3. Bump the `version` in frontmatter
4. Add a Change Log entry
5. Run `specsync check` to confirm

When you add new source files:

1. Add the file path to the relevant spec's `files` list
2. Add any new exports to the Public API table
3. Run `specsync check` to confirm

When you create a new module:

1. `specsync add-spec <name>` or `specsync generate` to scaffold
2. Complete the spec content
3. Run `specsync check` to validate

---

## 7. Compaction and Archival

As specs accumulate changelog entries and tasks get completed, companion files grow. Two commands handle this:

### Compact changelogs

```bash
specsync compact --keep 10              # keep last 10 entries per spec
specsync compact --keep 5 --dry-run     # preview what would be removed
```

Trims older changelog entries to prevent unbounded growth. Use `--dry-run` first to preview.

### Archive completed tasks

```bash
specsync archive-tasks                  # move completed tasks to archive
specsync archive-tasks --dry-run        # preview what would be archived
```

Moves completed checkboxes from `tasks.md` files to an archive section, keeping active work visible.

---

## 8. Cross-Project References

When modules depend on other repositories:

```bash
# In the dependency repo: publish a registry
specsync init-registry

# In your repo: reference the dependency
# In frontmatter: depends_on: ["corvid-labs/algochat@messaging"]

# Validate local refs
specsync resolve

# Validate cross-project refs (fetches from GitHub)
specsync resolve --remote
```

See [Cross-Project References](cross-project-refs.md) for the full setup.

---

## 9. CI Integration

### GitHub Actions

```yaml
- name: Validate specs
  run: specsync check --strict --require-coverage 80
```

See [GitHub Action](integrations/github-action.md) for the official action with caching and PR comments.

### Pre-commit hook

`specsync hooks install` sets up a pre-commit hook that runs `specsync check` before every commit. If specs are invalid, the commit is blocked.

### Recommended CI pipeline

```bash
specsync check --strict                  # no warnings allowed
specsync check --require-coverage 80     # enforce coverage threshold
specsync score --json                    # track quality over time
```

---

## 10. Working with AI Agents

SpecSync is designed for AI-assisted development. Three integration modes:

### MCP server (recommended)

```bash
specsync mcp
```

Exposes `specsync_check`, `specsync_generate`, `specsync_coverage`, `specsync_score` as native tools. Claude Code, Cursor, and Windsurf can call them directly. See [For AI Agents](integrations/ai-agents.md) for setup.

### Agent instruction files

```bash
specsync hooks install
```

Generates instruction files (`CLAUDE.md`, `.cursor/rules`, etc.) that tell AI agents to read specs before modifying code, update specs when changing APIs, and run validation after changes.

### JSON output for scripting

Every command supports `--json` (or `--format json`) for structured output. Pipe to an LLM for automated spec maintenance:

```bash
specsync check --json | your-agent-script
```

---

## Companion Files in Practice

The canonical spec plus four required companion files gives each module structured context beyond the technical contract. Projects can also enable the optional design companion.

### `<module>.spec.md` — The contract

The source of truth for what the module does and what it exports. SpecSync validates this against code. Keep it accurate — if the spec says `authenticate` exists, it must exist in the source files.

### `requirements.md` — The intent

Written by Product or Design. User stories, acceptance criteria, constraints, out-of-scope items. Helps developers and agents understand *why* the module exists, not just *what* it exports.

### `tasks.md` — The work

Checkboxes for outstanding work. Review sign-offs (Product, QA, Design, Dev). Helps teams track what's done and what's left. Use `specsync archive-tasks` to clean up completed items.

### `context.md` — The background

Design decisions, constraints, key files to read first, current status notes. The "tribal knowledge" file — things that aren't obvious from the code alone. Especially valuable for AI agents that need to understand *why* things are the way they are.

### `testing.md` — The evidence plan

Test strategy, requirement coverage, QA checks, fixtures, and edge cases. It records how the contract will be proved rather than only describing the intended implementation.

### `design.md` — The optional experience contract

Layout, component hierarchy, interaction states, accessibility expectations, and design tokens for changes where design artifacts are enabled.

---

## Common Workflows

### Adding a new module to an existing project

```bash
specsync add-spec payments             # scaffold spec + companions
# Edit specs/payments/payments.spec.md — complete Purpose, Public API, etc.
specsync check                          # validate
specsync coverage                       # confirm it shows up
```

### Reviewing spec drift in a PR

```bash
specsync diff --base main               # what changed since main
specsync check                          # any drift?
specsync check --fix                    # auto-stub new exports
# Review generated rows and finalize descriptions
```

### Bootstrapping specs for an existing project

```bash
specsync init                           # create config
specsync generate                       # deterministic local scaffolds
specsync agents install                 # agent refines them in its own trust boundary
specsync check                          # validate generated specs
specsync score                          # check quality
# Iterate: fix errors, improve low-scoring specs
specsync hooks install                  # set up agent instructions + hooks
```

### Onboarding a new team member

Point them to:
1. `specsync coverage` — what's specced and what isn't
2. The `specs/` directory — read the specs for their area
3. `specsync hooks install` — set up their local hooks
4. This guide — understand the workflow
