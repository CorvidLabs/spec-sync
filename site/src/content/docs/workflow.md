---
title: "Workflow Guide"
section: "Reference"
order: 2
---

End-to-end walkthrough of the verified SpecSync 6.0 change workflow.

---

## The Change Workflow

A pull request has an Archive check. While a change is still open the check fails, names the one change to archive first, and names one command. When every change is archived, the check says the pull request is ready to merge once the other checks are green.

`specsync change pr` archives the change when it is ready, pushes the branch, and opens the pull request. An open pull request for that branch is left in place. The command does not merge.

When that command is `specsync change ship`, the command archives the change and commits the archive. The next line says to push that commit, wait until the checks are green, and then merge. When it is not time, the same line names the one reason and the one command.

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

A change that will not ship is closed instead: `change close <id>... --actor <name> --reason "<why>"`
moves it, from any state before acceptance, into `.specsync/archive/closed/` with who closed it and
why, never reviewed, and frees its paths. `--replaced-by <id>` names the change that takes over.

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

The shared deterministic engine asks only unresolved questions and selects requirements, research, design, plan, tasks, context, testing, docs, or custom artifacts according to change type and risk. After those questions, and only when an affected module already has lessons or the project names a principles file, it asks which of them constrain the change. "none" is a complete answer. A change that has left draft without that answer is not sent back to the interview. Agent skills present the same questions conversationally.

A selected plan has to say what the change will do, what it will not do, the steps, the risks, and which existing constraints it honors before it can be approved. A bug fix or a documentation change that did not select a plan does not grow one.

Opening, showing, or asking the status of a change shows those lessons and that principles file, in text and in JSON. A module context that still holds only its generated scaffold is left out. Finalizing or shipping names the lesson bundle and still tells you to merge. Copying lessons into the spec is not required before the merge.

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

An approval shows what was approved, not who approved it: its digest binds the content, and `--actor` is a recorded claim that SpecSync does not authenticate. The same holds for `--reviewer`.

### Relayed decisions

When an agent records a decision a person made somewhere else, it says who decided, their exact words, where they said it and which agent is recording it:

```bash
specsync change approve add-passkeys --actor "Ada" \
  --words "Approve the passkey scope" \
  --where "team chat, 2026-11-05 21:40 MST, relayed by the release agent" \
  --recorded-by "release agent"
```

`change review` takes the same three flags. The record is stored as `relayed_by` (`decided_by`, `words`, `where`, `recorded_by`), with `decided_by` taken from `--actor` or `--reviewer`. All three flags go together: giving only some is refused, naming the missing ones, and a relayed approval needs an explicit `--actor`. `change status` prints it under the approval or review it belongs to:

```text
  Approval: Ada (a recorded claim, not authenticated)
    Relayed: Ada said "Approve the passkey scope" (team chat, 2026-11-05 21:40 MST, relayed by the release agent); recorded by release agent; a relayed claim, not authenticated: it records who is said to have decided, not proof that they did.
```

A relayed decision is a claim too. It satisfies no gate, and no digest covers it; the finalization review digest leaves it out. SpecSync 6.2.0 and older drop `relayed_by` when they rewrite a change, so a change that holds a relayed decision is recorded at workflow version 3, which those versions refuse with "was written by a newer SpecSync" instead of rewriting it (REQ-change-158). Version 3 is workflow v2 in every other way, and raising it stales nothing. Once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later; check-only uses (the GitHub Action and Trust pinned at 6.2.0) keep working. Relays on legacy workflow-v1 (portable 5.0.1) changes are not gated, and approve and review say so. If a hand-edited `relayed_by` cannot be read in an active change, the command names `specsync change repair-relays <id>`. It works whether or not the file is committed, which matters between `change review` and `change ship`, when neither review file is: it puts each damaged relay back from the last commit, or from its twin (`review.json` and the last review attempt are copies of each other), and otherwise removes only that `relayed_by`, never the approval or review attempt it belongs to, and says what it did. No digest reads a relay, so a repair stales nothing. An archived package is committed by finalize, so there the command names `git checkout HEAD -- <file>`.

## 3. Check, Review, and Finalize

```bash
specsync change check add-passkeys --commit
# run the local verify lane: archive-first CI does not test a tip with an open change
# open or update the PR, then complete ordinary PR review
# after a human reviewer passes the change package, diff, spec delta, and evidence:
specsync change review add-passkeys --reviewer "Ada Reviewer"
specsync change finalize add-passkeys
# do not commit between review and finalize; finalize commits the archive move
# push the archive commit, wait for required checks, then merge on GitHub
```

A change whose `tasks.md` still has an open box (`- [ ]`) is refused by `change check`, `change review` (a passing verdict), `change finalize` and `change ship`, before any of them writes a record. That includes a box reopened after a passing check. Ticking a box is outside the approval and verification digests, so closing a task never costs a fresh approval or a fresh check. The refusal names each open task by line and the exact command to run once the work is done:

```text
error: tasks.md contains incomplete tasks: finish the 1 open task in `.specsync/changes/add-passkeys/tasks.md` (line 4: "Prove the fix.") and mark it `[x]`, then re-run `specsync change check add-passkeys --commit`
```

Finish the work, tick the box, and run that command. `change status` and `change ship-status` name the same open tasks before their next step.

CI is archive-first. The `Archive` check runs before anything else and every other job depends
on it, so a pushed tip with any open change fails `Archive` and runs nothing else. Verification
and review are local; the full CI lane runs once, on the archived tip. If that lane fails, push a
fix commit on top with specs kept in sync (`specsync check --strict`). A fix that changes the
approved scope needs a new change.

`change check` applies approved semantic deltas and compares specs to code in-process for
**this change only**. It does not run the project's tests and does not re-validate archived
terminal evidence — archives are history. Use `change audit` for project health over **active**
workspaces and living specs. CI owns `cargo test` / `swift test` / `bun test`. Explicit `--strict`, project policy, and deterministic release/security classification add
validators to the same scoped evidence path. Source, test, configuration, or contract edits stale
verification; implementation edits after scoped review stale the review. Files under a
`verification_ignored_paths` directory are the exception: `change status` and `change check` name
them under "Verification ignores" (see the SDD policy in the configuration guide). `change status` prints the
exact `change review` command when this human implementation review is the next required action; `change
review` records the completed review and does not replace the repository's ordinary PR review.

An unsuppressed validator warning in a module the change declares, such as an undocumented
export, fails `change check` on its first run as well as on later ones. Finalization already
refused it. This does not depend on the configured `enforcement`, which still governs
`specsync check` only. The failed attempt is recorded: `verification.json` reports
`passed: false` with an entry naming `specsync change check <id> (effective contract: <module>)`,
so `change status` points back to `change check`.

Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate. The scope approver may also perform the implementation review.

`change finalize` requires current verification and scoped review, writes finalization evidence,
and moves the package to `.specsync/archive/changes/YYYY-MM-DD-<id>/`. That command commits
the move as `chore(lifecycle): archive <id>`. Run `review` and `finalize` (or `ship`)
consecutively without an intervening commit. Push the archive commit and wait for required
checks. Never merge while a change on that PR remains active.
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

### Send a change back for re-scoping

When an approved, implementing or verifying change turns out to have the wrong scope (for
example a spec split moved the requirements it edits into another module), send it back to draft
instead of deleting its workspace:

```bash
specsync change send-back add-passkeys \
  --actor "Ada Reviewer" \
  --reason "The session requirements moved to the sessions spec"
specsync change answer add-passkeys affected_specs auth,sessions
# rewrite the deltas for the new scope
specsync change approve add-passkeys --actor "Ada Reviewer"
specsync change check add-passkeys --commit
```

The person and the reason are appended to `approvals.json` under `send_backs`. The definition
approval recorded before the send-back is kept but no longer counts, so the new scope needs a
fresh `change approve`, even when it did not change. A committed send-back cannot be deleted to
bring that approval back: check refuses a ledger that lost or changed a send-back held by any
earlier commit, and `specsync change audit` reports it.

Spec text the change already applied stays in place, and the send-back lists it under
`applied_items`, REMOVED items included, with each block's text from before the change. It also
lists the modules whose Change Log carries the change's row. After a send-back, `change check`
converges on the canonical tree: a kept item that is already applied (a removed block still gone,
an added or modified block with the same body) is not applied again, a kept ADDED item with a new
body replaces the change's own earlier body, and the rest is applied. If the new scope drops an
item that was already applied, the spec would keep that edit with no change owning it, so
`change check` and `change accept` refuse and say what to restore for each one, for example
`specs/auth/auth.spec.md` section `Purpose`: take out a block the change added, or put back the
pre-change text (quoted in the message) of a block it modified or removed. A module dropped from
`affected_specs` is refused while its Change Log still has the change's row; remove that row and
restore its `version:`. Then check again (or keep the item in the delta). Accepted or archived
changes use `change reopen` instead.

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
# rerun the local verify lane and complete human review (CI waits for the archive tip)
specsync change review add-passkeys --reviewer "Ada Reviewer"
specsync change finalize add-passkeys
# finalize commits the archive; push it and wait for required checks before merging
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

### Close a change that merged long ago

A change that merged without being archived blocks every later finalize, and its evidence may no
longer be checkable. `specsync doctor` lists it with its fix. When re-checking is not possible or not
worth it, a person can close it late:

```bash
specsync doctor --fix --dry-run --close-late <id> --actor <name> --reason "<why>"
specsync doctor --fix --close-late <id> --actor <name> --reason "<why>"
```

The package moves byte for byte into `.specsync/archive/closed-late/`, with a `late-close.json` that
records who, when and why, the commit that merged it, and `verified: false`. It never counts as
checked: `change status` says it was closed late, and every lifecycle verb refuses it. Only a change
whose package is on the remote default branch can be closed late. SpecSync 6.0 to 6.2 never read
that folder. If the change belonged to an acknowledged sequence collision, doctor takes it out of that
acknowledgement in `.specsync/change-sequence.json` (the whole acknowledgement once fewer than two
members remain) and keeps the acknowledgement as it stood in the record; where meaningful
files need an open change, the pull request's change must declare that file.

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

After upgrading to a version containing the hook quoting fix for [#782](https://github.com/CorvidLabs/spec-sync/issues/782), refresh an existing pre-commit hook from the same project root:

```bash
specsync hooks uninstall --precommit
specsync hooks install --precommit
```

Installing alone leaves an existing managed block unchanged. The uninstall removes this project's managed block while preserving user content and other projects' blocks. For a nested project, pass the same `--root <project>` to both commands. Until refreshed, hooks generated by an older version retain their old contents, including the unsafe failure diagnostic for paths containing shell substitutions or apostrophes.

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


### Historical acknowledgement is a different proof basis

A stored receipt can be authentic even when its original file transition cannot be reconstructed from the definition base. Normal succession continues to refuse that case. The opt-in `change reconcile-history` scope, available from 6.2.0, preserves and authenticates committed accepted packages and explicitly labels the old transitions unproven. It requires new definition approval and fresh current verification and scoped review. It is administrative historical reconciliation, not a claim that old product behavior was accepted again or that missing historical evidence has been recovered. Its versioned evidence and status/finalize disclosure retain that distinction after archival.

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
