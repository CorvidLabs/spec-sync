---
title: "Semantic Deltas"
section: "Reference"
order: 3
---

A **semantic delta** is the machine-applicable record of how a change modifies canonical spec truth. Deltas are what let the 6.0 lifecycle keep spec edits inside the same verified workflow as code edits: nothing reaches the canonical spec except through a delta that was approved, materialized by `change check`, reviewed, and finalized.

---

## Where Deltas Live

Each change workspace carries one delta file per affected spec module:

```text
.specsync/changes/add-passkeys/deltas/auth.md
```

The file name must match the module name of an affected spec. Definition approval fails until the set of delta modules **exactly matches** the change's affected specs:

```text
semantic delta modules must exactly match affected specs (missing: auth; extra: none)
```

Documentation-only changes that touch no spec contract can opt out with `--no-spec-change` and a recorded rationale (see [Workflow](workflow.md)).

---

## Delta File Structure

A delta file contains `## ADDED`, `## MODIFIED`, and `## REMOVED` sections. Each section holds typed blocks:

| Block | Meaning |
|:------|:--------|
| `### REQUIREMENT <REQ-id>` | A normative requirement, with `SHALL` prose and acceptance criteria |
| `### SPEC SECTION <section name>` | Markdown content targeting a named canonical spec section (e.g. `Public API`) |

- **ADDED** — introduces a new requirement or a new section whose heading does not yet exist.
- **MODIFIED** — replaces the referenced content with the block's full intended content.
- **REMOVED** — retires a previously canonical requirement or section content.

Lines beginning with `## ` and `### ` are reserved for delta operations and items. Inside a block body, use `####` or a deeper heading level when structured Markdown needs a heading.

---

## Requirement Blocks

Requirement IDs use `REQ-<module>-<number>`. Replace underscores in the affected module name with hyphens, so a delta for `user_auth` uses IDs such as `REQ-user-auth-004`.

```markdown
## ADDED

### REQUIREMENT REQ-auth-004

The system SHALL let a signed-in user revoke every active session.

Acceptance Criteria
- Calling the revocation endpoint invalidates every session owned by that user.
```

Once finalized, requirement IDs remain reserved; removing one creates a permanent tombstone that prevents later reuse. During `change check`, every added or modified requirement ID in the delta must have **test or declared evidence** bound to it, or verification fails:

```text
requirement evidence missing for REQ-auth-004
```

Bind evidence by naming the exact requirement ID in the change's `testing.md` artifact or in a detected test file. `change check` then compares specs to code; it does not run the project's tests (see [Workflow](workflow.md)).

---

## Spec Section Blocks

```markdown
## MODIFIED

### SPEC SECTION Public API

| Name | Description |
|------|-------------|
| `create_session` | Creates a new session token for a signed-in user. |
| `revoke_all_sessions` | Revokes every session owned by the given user. |
```

For `MODIFIED` and `REMOVED`, the section name must match a section heading in the canonical spec.
For `ADDED`, the heading must not exist yet; use `MODIFIED` to change an existing section. A
`MODIFIED` block carries the section's **full intended content after the change** — when multiple
changes touch the same section in sequence, each delta restates the complete resulting table rather
than only its own additions.

Because a `MODIFIED` block replaces the whole block, the definition approval also records the
canonical text each `MODIFIED` and `REMOVED` block was approved against. If another change rewrites
that same block on the base branch and it is merged in before this change lands, `change check`
and `change audit` refuse rather than overwrite the newer text, and name the block that moved:

```text
semantic delta for `auth` was approved against canonical text that has changed since (auth: REQ-auth-001); applying it would overwrite text this change never approved, ...
```

Rewrite the delta body onto the current canonical text, then run `change approve` and `change check`
again. Acceptance refuses the same way. Re-approving a body you have not rewritten is refused too,
because it would record the newer text as the base and let the old body overwrite it. Approvals
recorded before SpecSync carried this field keep their earlier behaviour, but once a change has
recorded these bases, a later approval cannot drop them.

A delta cannot target the `## Change Log` section. The table is frozen, and each change is recorded
in its own history fragment, so `change approve`, `change check`, verification and acceptance refuse an `ADDED`,
`MODIFIED` or `REMOVED` `SPEC SECTION Change Log` item and name it. A change archived before this
rule is not re-judged.

---

## The Effective Contract

While a change is active, the `change` commands validate against the **effective contract**: the canonical spec composed with all approved, non-conflicting deltas. Plain `specsync check` reads only the canonical files on disk; it does not consult change workspaces.

Practical consequence: an approved delta must compose cleanly with the canonical spec and with every other active delta before it can be applied. The canonical spec itself is updated when `change check` materializes the approved delta — after that, `specsync check` sees the new contract too.

---

## Conflicts and Ordering

Two active changes that touch the same delta surface produce deterministic validation errors — conflicting edits cannot silently interleave. Ordering tools:

- `specsync change depend update-ui add-passkeys` declares that `update-ui` builds on `add-passkeys`.
- When several active deltas compose into one effective contract, declared dependencies determine their topological order; change IDs provide the deterministic order among otherwise independent changes.

Fix classification edits to accepted metadata with `change correct` rather than editing delta files after acceptance — fresh classification edits no longer conflict with sibling accepted deltas, but stale accepted content still routes through the audited correction chain.

### A requirement lost without a REMOVED delta

A branch that lost a canonical requirement is refused too. `change check`, verification, `change
accept`, `change finalize` and `change ship` compare the requirement ids in every `requirements.md`
(and legacy `<module>.req.md`) with those at the merge-base of `HEAD` and the default branch, or at
the change's base commit when there is no default branch. An id that is gone from every requirements
file is refused, by name, unless an approved `## REMOVED` item of a change on the branch names it. A
requirement moved to another file under the same id is not lost; a renumbered one is lost under its
old id, because a requirement id is permanent. To put a lost block back, `git show <commit>:<file>`
prints it as the merge-base held it; to remove it on purpose, name it in a `## REMOVED` delta and
approve the change again. `change audit` reports the same refusal, and `change ship-status` lists it
as a blocker and exits 1 (REQ-change-169, #1081).

The check reads no requirements under `.git`, `.specsync` (change workspaces and archives hold
`requirements.md` artifacts, not canonical requirements) or any `exclude_dirs` directory, even when
`specs_dir` is the project root, and a change workspace it cannot read accounts for no removal. When
Git cannot answer, for example in a shallow clone that lacks the merge-base, the check refuses and
says so: repair the repository or fetch the default branch (`git fetch --unshallow origin` in a
shallow clone, `git fetch origin` otherwise), then run `change check` again. With no default branch
and no base commit in `HEAD`'s history, as in a depth-1 CI checkout, it is skipped with a note.

---

## Check and Finalization: Atomic Application

`specsync change check` applies the approved delta to the canonical spec **atomically** before
targeted verification. After ordinary and scoped PR review, `specsync change finalize` binds that
implementation and moves the package into the dated archive in the same PR:

- the change is recorded in its own history fragment, `specs/<module>/history/<change-id>.md`
  (date, change id and title); the spec's `version` and its Change Log table are not touched,
- requirement blocks land in the canonical `requirements.md`.

Because each change writes its own history file, two changes to one module never edit the same
line of its spec for their history, so they merge with plain Git, including GitHub's merge button,
which runs no merge driver. A Change Log table written by an earlier SpecSync stays exactly as it
is; `specsync history <module>` prints it followed by one row per fragment.

SpecSync 6.3.0 and older do not know history fragments. Their `check` ignores them, but their
`change audit` fails on a tree that has one (it reads the fragment as a path no active change
covers), and their `change check` on a change recorded by a fragment appends a Change Log row and
bumps `version:` as well, which a newer `check` reports as a change recorded twice. A repository
that pins SpecSync must move the pin in the same pull request that first ships a change with
history fragments.

Application is protected against double-apply: if a block already exists in the canonical spec
(for example because it was hand-applied outside the lifecycle), checking fails rather than
duplicating it:

```text
cannot add existing block REQ-sessions-001
```

Historical `change reopen` and `change correct` repair paths never reapply an already-canonical
delta.

---

## Full Example

A complete delta for a change documenting a `sessions` module:

```markdown
## ADDED

### REQUIREMENT REQ-sessions-001

The system SHALL let a signed-in user revoke every active session.

Acceptance Criteria
- tests/test_sessions.py asserts revoke_all_sessions revokes every owned session.

### REQUIREMENT REQ-sessions-002

The system SHALL create a unique session identifier for a signed-in user.

Acceptance Criteria
- tests/test_sessions.py asserts create_session returns a session token.

## MODIFIED

### SPEC SECTION Public API

| Name | Description |
|------|-------------|
| `create_session` | Creates a new session token for a signed-in user. |
| `revoke_all_sessions` | Revokes every session owned by the given user. |
```

> **Note:** Runnable end-to-end usage — creating the change, answering the interview, approving, checking, reviewing, and finalizing — is demonstrated in `examples/sdd-five-epics/run.sh` and `examples/sdd-lifecycle/run.sh`.

---

## See Also

- [Workflow](workflow.md) — the full verified lifecycle
- [Spec Format](spec-format.md) — canonical spec structure deltas apply against
- [CLI Reference](cli.md) — `change` command surface
