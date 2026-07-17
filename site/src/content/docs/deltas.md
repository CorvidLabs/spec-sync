---
title: "Semantic Deltas"
section: "Reference"
order: 3
---

A **semantic delta** is the machine-appliable record of how a change modifies canonical spec truth. Deltas are what let the 5.x lifecycle keep spec edits inside the same verified workflow as code edits: nothing reaches the canonical spec except through a delta that was approved, verified, and accepted.

---

## Where Deltas Live

Each change workspace carries one delta file per affected spec module:

```text
.specsync/changes/CHG-0001-add-passkeys/deltas/auth.md
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

- **ADDED** — introduces a new requirement or new section content.
- **MODIFIED** — replaces the referenced content with the block's full intended content.
- **REMOVED** — retires a previously canonical requirement or section content.

---

## Requirement Blocks

```markdown
## ADDED

### REQUIREMENT REQ-auth-004

The system SHALL let a signed-in user revoke every active session.

Acceptance Criteria
- Calling the revocation endpoint invalidates every session owned by that user.
```

Requirement IDs introduced through deltas are permanent. Before acceptance, every requirement ID in the delta must have **test or declared evidence** bound to it, or acceptance fails:

```text
requirement evidence missing for REQ-auth-004
```

Evidence is typically bound through the change's testing artifact and the configured `verification_commands` (see [Workflow](workflow.md)).

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

The section name must match a section heading in the canonical spec. A `MODIFIED` block carries the section's **full intended content after the change** — when multiple changes touch the same section in sequence, each delta restates the complete resulting table rather than only its own additions.

---

## The Effective Contract

While a change is active, `specsync check` validates code against the **effective contract**: the canonical spec composed with all approved, non-conflicting deltas.

Practical consequence: an export documented only in an approved delta already counts as documented during implementation — validation reports reflect the composed view, not just the canonical files. The canonical spec itself is updated only at acceptance.

---

## Conflicts and Ordering

Two active changes that touch the same delta surface produce deterministic validation errors — conflicting edits cannot silently interleave. Ordering tools:

- `specsync change depend CHG-0002 CHG-0001` declares that CHG-0002 builds on CHG-0001.
- When several active deltas compose into one effective contract, they apply in a deterministic order (delivery base commit, then change id).

Fix classification edits to accepted metadata with `change correct` rather than editing delta files after acceptance — fresh classification edits no longer conflict with sibling accepted deltas, but stale accepted content still routes through the audited correction chain.

---

## Acceptance: Atomic Application

At closing approval, `specsync change accept` applies the delta to the canonical spec **atomically** — no partial writes — and stamps provenance:

- the canonical spec's `version` increments,
- the Change Log records the acceptance,
- requirement blocks land in the canonical `requirements.md`.

Application is protected against double-apply: if a block already exists in the canonical spec (for example because it was hand-applied outside the lifecycle), acceptance fails rather than duplicating it:

```text
cannot add existing block REQ-sessions-001
```

Neither `change reopen` nor `change correct` reapplies an already-canonical delta.

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

> **Note:** Runnable end-to-end usage — creating the change, answering the interview, approving, verifying, and accepting — is demonstrated in `examples/sdd-five-epics/run.sh` and `examples/sdd-lifecycle/run.sh`.

---

## See Also

- [Workflow](workflow.md) — the full verified lifecycle
- [Spec Format](spec-format.md) — canonical spec structure deltas apply against
- [CLI Reference](cli.md) — `change` command surface
