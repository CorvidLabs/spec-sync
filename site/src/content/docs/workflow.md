---
title: "Workflow Guide"
section: "Reference"
order: 2
---

End-to-end walkthrough of the verified SpecSync 6.0 change workflow.

---

## The Change Workflow

Every new delivery follows one predictable path:

```text
new → one scope approval → implement → check → PR review → finalize → GitHub merge
```

| State | What happens | Key commands |
|:------|:-------------|:-------------|
| **Draft** | Deterministic interview selects scope and adaptive artifacts | `change new`, `change answer` |
| **Approved** | A human approves the scope/definition digest once | `change approve` |
| **Implementing** | Code, canonical specs, and tests follow the approved package | `change check` |
| **Verifying** | Targeted evidence is current and one scoped PR review is required | `change check`, ordinary PR review, `change review` |
| **Archived** | Finalization records closure and moves the package to dated history in the same PR | `change finalize` |

Module maturity (`draft → review → active → stable → deprecated → archived`) remains separately available through `specsync lifecycle`.

## 1. Create and Interview

```bash
specsync change new "Add passkeys" --spec auth --path src/auth.rs --json
specsync change answer add-passkeys acceptance_criteria \
  "A registered passkey authenticates the user" --json
specsync change answer add-passkeys public_contract yes --json
specsync change answer add-passkeys architecture_risk yes --json
```

Acceptance criteria preserve the submitted prose as one criterion, including commas and line breaks. To submit more than one criterion, pass an explicit JSON string array such as `'["Passkey login works", "Recovery remains available"]'`. Scope questions such as `affected_specs` and `affected_paths` continue to accept comma- or newline-separated lists.

The shared deterministic engine asks only unresolved questions and selects requirements, research, design, plan, tasks, context, testing, docs, or custom artifacts according to change type and risk. Agent skills present the same questions conversationally.

New changes use a slug derived from their description, such as `add-passkeys`. Use the ID
returned by `change new`. A repeated description is refused and names the existing change;
choose a distinct description for distinct work. The sequence ledger preserves historical
numeric identities only. It no longer allocates new change numbers and must not be hand-edited.

## 2. Approve and Implement

Complete selected artifacts and semantic deltas, then obtain explicit human approval:

```bash
specsync change approve add-passkeys --actor "Ada"
```

Requirements use stable IDs, normative SHALL statements, and acceptance criteria. Only a change to the approved stable intent, contract, acceptance criteria, or affected scope requires renewed human approval. Implementation details, tests, evidence, canonical delta materialization, and lifecycle metadata preserve that approval while automated verification and the scoped human review are refreshed.

## 3. Check, Review, and Finalize

```bash
specsync change check add-passkeys --commit
# push the product tip after local gates and wait for required checks
# open or update the PR, then complete ordinary PR review
# after a human reviewer passes the change package, diff, spec delta, and evidence:
specsync change review add-passkeys --reviewer "Ada Reviewer"
specsync change finalize add-passkeys
# do not commit between review and finalize
# commit and push the archive result, wait for required checks, then merge on GitHub
```

`change check` applies approved semantic deltas and compares specs to code in-process for
**this change only**. It does not run the project's tests and does not re-validate archived
terminal evidence — archives are history. Use `change audit` for project health over **active**
workspaces and living specs. CI owns `cargo test` / `swift test` / `bun test`. Explicit `--strict`, project policy, and deterministic release/security classification add
validators to the same scoped evidence path. Source, test, configuration, or contract edits stale
verification; implementation edits after scoped review stale the review. `change status` prints the
exact `change review` command when this human implementation review is the next required action; `change
review` records the completed review and does not replace the repository's ordinary PR review.

Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate. The scope approver may also perform the implementation review.

`change finalize` requires current verification and scoped review, writes finalization evidence,
and moves the package to `.specsync/archive/changes/YYYY-MM-DD-<id>/`. Run `review` and
`finalize` (or `ship`) consecutively without an intervening commit. Commit the archive result
on the same PR and wait for required checks. Never merge while a change on that PR remains active.
GitHub protection and required workflows enforce hosted review and provenance policy where
configured; the local reviewer label alone does not supply that authentication.

Historical `start`, `verify`, `accept`, `archive`, `correct`, and `correct-owner`
commands remain available to validate or repair older two-approval evidence. They are compatibility
surfaces, not steps in the new-change workflow.

### Clearing context between steps

An agent session that clears its context at the wrong moment loses the intent behind uncommitted
work; one that clears at a recorded boundary resumes cleanly. `change status`, `change show`, a
passing `change check`, `change approve`, `change review`, and `change finalize` each end with one
`Handoff:` line that says which of the two you are at:

```text
  Next: run `specsync change check add-passkeys`
  Handoff: conditional — uncommitted edits sit under this change's paths, and nothing on disk records why they were made. Before clearing: commit the work in progress; or write its intent and open ends into `.specsync/changes/add-passkeys/change.md`
```

- `safe` — everything the next session needs is recorded. The reason says where it resumes:
  a freshly approved definition resumes at `change check`; current verification resumes at the
  human implementation review or at finalize; workflow-v2 acceptance and the archive need nothing further.
- `conditional` — the lifecycle is consistent, but something lives only in this session. A
  Draft is never `safe`: answer the open questions and approve first, or write the undecided
  points into `change.md`. Uncommitted edits under the change's `affected_paths` are
  `conditional` until they are committed or their intent is written down. Evidence under
  `.specsync/` alone never counts — `review.json` is uncommitted between `review` and `finalize`
  by design.
- `not yet` — a gate is broken and the next session would not be able to see why: a stale
  approval digest, a frozen sequence ledger, an invalid correction ledger, or stale legacy
  terminal evidence. The line names the repair.

Resume with `specsync change status <id>`. `--json` carries the same verdict as
`summary.handoff` (`readiness`, `reason`, `resume`, `before_clearing`) wherever a change summary
is rendered, and as `handoff` on the approve transition. The installed agent skill tells agents to
clear context only on `safe` and to do what `Before clearing:` names first.

## 4. Recover accepted or archived evidence

`reopen` also supports workflow-v2. An interrupted finalization can leave an accepted record
with a terminal finalization approval; an already archived change can also need recovery when
its accepted delivery inputs become stale. For an eligible record whose approved definition
is unchanged, use the audited recovery path:

```bash
specsync change reopen add-passkeys \
  --actor "Ada Reviewer" \
  --reason "Review correction changed a scoped delivery input after finalization"
specsync change check add-passkeys --commit
# push the corrected product tip, wait for required checks, and complete human review
specsync change review add-passkeys --reviewer "Ada Reviewer"
specsync change finalize add-passkeys
# commit and push the archive result; wait for required checks before merging
```

An archived package moves back to the active workspace on a successful reopen. The audit
preserves its prior state and superseded terminal approval. A refused reopen restores the
archive. Consult `change status <id>` first: missing terminal evidence or an unchanged current
record does not justify reopening, and a changed definition requires its own approval path.
Do not add legacy `accept` to this workflow-v2 sequence.

### Legacy workflow-v1 recovery

The numeric IDs and `verify` / `accept` commands below describe historical workflow-v1 records.
Consult `change status <id>` before repairing that evidence.

If final review changes a governed source, test, configuration, policy, or contract input after acceptance, strict checking correctly rejects the stale closing evidence. Do not edit lifecycle JSON or archive the active workspace. Record an audited transition instead:

```bash
specsync change reopen CHG-0001-add-passkeys \
  --actor "Ada Reviewer" \
  --reason "Final review changed the authentication policy and tests"
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys --actor "Ada Reviewer"
```

Eligible recovery includes stale accepted delivery inputs and supported historical cases where the old evidence cannot be reconstructed. The CLI checks the recorded history and reports eligibility; missing or arbitrary evidence is not permission to bypass validation. It moves the change back to `verifying`, embeds the prior verification and superseded closing approval in append-only audit history, and leaves strict CI red until a fresh verification succeeds. Reacceptance records a new closing approval without applying the already-canonical semantic delta a second time. Use global `--json` to receive the deterministic change and versioned audit objects.

Squash-integrated changes and changes partially superseded by later canonical work may also reopen when current Git history records their accepted state or later recorded canonical changes govern every affected contract surface. The unchanged definition, passed evidence, valid closing approval, an eligible recovery cause, explicit actor, and audit reason remain mandatory; copied or arbitrary off-history evidence is rejected.

The reopened definition must remain identical to the contract that originally applied the canonical delta. If review requires new or changed requirements, deltas, or other definition artifacts, create a new change workspace; reacceptance fails closed instead of silently ignoring those edits.

If re-verification proves that an already-scoped production input had another canonical owner omitted from the historical affected-spec list, record that exact ownership correction after reopening:

```bash
specsync change correct-owner CHG-0001-add-passkeys \
  --path src/auth.rs \
  --spec auth-policy \
  --actor "Ada Reviewer" \
  --reason "The historical definition omitted the canonical auth-policy owner"
specsync change approve CHG-0001-add-passkeys --actor "Ada Reviewer"
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys --actor "Ada Reviewer"
```

`correct-owner` is intentionally exact and additive. The path must already be inside the original delivery scope, the named module's current canonical spec must list that source file, and the change must be already applied and in `verifying` through an audited reopen. The command cannot add paths, affected specs, requirements, or semantic deltas. It preserves prior approval and reopen evidence, makes the definition approval stale, and requires fresh approval, verification, and closing approval. Reacceptance adds the corrected module only to that manifest entry and does not apply the canonical delta again. If the needed change is a real semantic rescope, create a successor change instead.

Historical succession checks preserve exact coverage and approval requirements. For new
slug-based changes, use the supported `change supersede` workflow to record predecessor,
path, and module obligations before approval; do not infer succession from numeric ordering.
A partial, draft, failed, or stale record is not proof that predecessor obligations were met.

## 5. Legacy workflow-v1 recovery: correct accepted classification

Use `change correct` when the accepted delivery evidence is current but review proves that the original `public_contract` or `architecture_risk` answer was wrong:

```bash
specsync change correct CHG-0001-add-passkeys architecture_risk yes \
  --actor "Ada Reviewer" \
  --reason "The persistence boundary makes this an architectural change"
# complete any newly selected research/design/plan/tasks/testing artifacts
specsync change approve CHG-0001-add-passkeys --actor "Ada Reviewer"
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys --actor "Ada Reviewer"
```

Correction is intentionally narrower than editing an accepted definition. It supports only the two closed yes/no classification fields. The original answers, selected artifacts, approvals, and verification remain unchanged and inspectable; `corrections.json` records the actor, reason, values, portable view digests, prior evidence, and artifacts added by each correction. Effective artifacts are monotonic, so changing a value back to `no` never removes scrutiny already recorded.

Correction moves `accepted` to `verifying` and requires a fresh definition approval before verification plus a fresh closing approval before reacceptance. A second correction is available only after the first is reaccepted. Because canonical application remains recorded, reacceptance never applies or version-bumps the semantic delta twice. Use `change reopen` instead when the definition is unchanged and only governed delivery inputs made accepted evidence stale.

The repository includes executable examples for a [complete lifecycle](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-lifecycle), [ordered concurrent changes](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-concurrent-changes), and a [five-epic product evolution](https://github.com/CorvidLabs/spec-sync/tree/main/examples/sdd-five-epics). Each creates a disposable Git project and runs the real CLI end to end.

---

## Project Setup

### Initialize a project

```bash
specsync init
```

This creates `.specsync/config.toml`, `.specsync/sdd.json` with the change workflow **off**, and
the change/archive directories. It does not detect test commands or start a change interview.
`specsync change adopt` turns the workflow on: it sets `enabled: true` and leaves every other
policy field and all historical evidence untouched, then routes subsequent changes through this
workflow. Commit and integrate every active legacy change first: adoption refuses to publish a
partial migration when a v1 record is absent from the trusted comparison cutoff.

### Install hooks and agent instructions

```bash
specsync hooks install
```

This installs:
- **Agent instructions** — `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md` — so AI coding tools know to respect specs
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

The spec file is validated bidirectionally against code. Companion files provide structured context for humans and AI agents, and strict mode also rejects known generated scaffold markers left in `context.md`, `requirements.md`, `testing.md`, `tasks.md`, or `design.md`. Diagnostics identify the artifact path and line; fenced examples containing a marker are ignored.

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
| Spec documents 'foo' but no matching export found in source | Remove `foo` from the spec, or add it to the code |
| Undocumented export 'bar' from src/mod.ts | Add `bar` to the Public API table |
| Source file not found: src/old.ts | Update the `files` list in frontmatter |
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

Shows file and LOC coverage — what percentage of your source code has a spec. HTML, HTM, and CSS are auto-detected and measured by default alongside language sources, so zero-config static repositories produce real covered and uncovered counts rather than disappearing from the denominator. Use `--json` to get machine-readable output with `uncovered_files` sorted by size, so you can prioritize the largest gaps.

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

See [GitHub Action](integrations/github-action.md) for the official action with checksummed download and optional PR comments.

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
specsync mcp                 # read-only tools
specsync mcp --allow-write   # opt in to root-confined init/generate tools
specsync mcp lock --write    # commit MCP tool-contract lock
specsync mcp diff            # fail-closed lock drift check (CI)
```

The default server exposes validation, coverage, listing, scoring, and issue-verification tools.
`--allow-write` additionally exposes deterministic generation and initialization at the configured
server root. Claude Code, Cursor, and Windsurf can call them directly. See [For AI Agents](integrations/ai-agents.md) for setup and confinement details.

### Agent instruction files

```bash
specsync hooks install
```

Generates instruction files (`CLAUDE.md`, `.cursorrules`, `AGENTS.md`, etc.) that tell AI agents to read specs before modifying code, update specs when changing APIs, and run validation after changes.

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
