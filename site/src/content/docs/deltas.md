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

When main's edit and the change's own edit to a `## MODIFIED` requirement do not overlap word by
word, `change check` proposes the rebase itself. It reads main's text for the requirement from Git
at the merge-base of HEAD and the default branch (`origin/HEAD`, else `origin/main` or
`origin/master`), never from the working tree, takes the approval-time text from that branch's
earlier history, merges the three word by word, writes the merge into the delta, and refuses:

```text
semantic delta for <id> now holds a proposed rebase over main's edit, and nothing else was written: auth: REQ-auth-001 (main's text at 1a2b3c4d5e6f). A proposal is not approved wording: read it in the delta, approve it with `specsync change approve <id>`, then run `specsync change check <id>`
```

A proposal writes only the delta. The canonical requirement and `approvals.json` stay as they were,
and `change status` shows the delta as changed after approval, so nothing reaches the canonical
requirement until a person reads the merge and approves it; the next `change check` then applies it
like any approved body. The requirement in the working tree must be main's text, the change's text
or already their merge: a clean Git merge and a conflict resolved to either side get the same
proposal. A requirement main has edited that cannot be proposed is refused even when the branch
kept the change's own text, because keeping it would drop main's edit. The refusal names the cause
and the way on for each requirement:

- the edits overlap: the same words changed on both sides, a word both sides deleted at one place
  that one side adds at another (as when both move one word), or a merge that would lose or
  duplicate a word;
- main deleted the requirement, or edited its heading;
- the working tree holds some other text, such as a sentence added by hand on the branch;
- the requirement is longer than the merge reads, 4,000 words a side;
- a default branch exists and Git cannot read main's text (repair the repository or
  `git fetch origin`, then check again).

For all but the last, rewrite the body onto main's text by hand and approve it. With no default
branch, and in CI, nothing is proposed and such a requirement materializes as before. Spec
sections, `## REMOVED` items and workflow-v1 changes keep the refusal above.

Otherwise, rewrite the delta body onto the current canonical text, then run `change approve` and
`change check` again. Acceptance refuses the same way. Re-approving a body you have not rewritten is refused too,
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
`change audit` and `change audit --strict` fail on a branch whose changes add a fragment, at the
checked tip and at the archive tip (they read the fragment as a path no active change covers); on
a branch that only holds fragments already merged they pass. Their `change check` on a change
recorded by a fragment appends a Change Log row and bumps `version:` as well, which a newer
`check` reports as a change recorded twice. A repository
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
