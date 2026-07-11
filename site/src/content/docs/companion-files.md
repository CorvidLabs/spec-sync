---
title: "Companion Files"
section: "Reference"
order: 3
---

Every canonical module spec lives beside focused companion files. Together they preserve product intent, work state, architectural context, and verification evidence without turning the executable contract into a project notebook.

```text
specs/auth/
├── auth.spec.md
├── requirements.md
├── tasks.md
├── context.md
├── testing.md
└── design.md        optional
```

## Shared frontmatter

Companions identify their canonical spec:

```markdown
---
spec: auth.spec.md
---
```

The filename, `spec` value, and module directory must agree. Companion files are permanent project context and should remain useful after the immediate change is complete.

## `requirements.md`

Requirements describe durable outcomes, not implementation tasks. New normative requirements use stable module-scoped IDs, a SHALL statement, and explicit acceptance criteria.

```markdown
---
spec: auth.spec.md
---

## Requirements

### REQUIREMENT REQ-auth-004

The authentication module SHALL let a signed-in user revoke every active session.

Acceptance Criteria

- Revocation invalidates every current refresh token owned by the user.
- A subsequent refresh attempt fails with the documented session error.
- Revoking an account with no sessions succeeds without creating state.

## Constraints

- Token material is never written to logs.

## Out of Scope

- Deleting the user account.
```

Do not check requirements off. They remain true after delivery. Change or remove them through a reviewed semantic delta so history and tombstones remain traceable.

## `tasks.md`

Tasks describe work, not truth. Keep current delivery work separate from later roadmap or debt.

```markdown
---
spec: auth.spec.md
---

## Tasks

- [x] Add session-store bulk revocation.
- [x] Add requirement-level regression coverage.

## Post-5.0 Roadmap

- [ ] Add administrator-initiated revocation.

## Post-5.0 Test Debt

- [ ] Add the long-running multi-region consistency fixture.
```

Verification requires selected change-workspace tasks to be complete. Canonical module task files may retain clearly categorized roadmap, test-debt, or manual work.

Use `specsync archive-tasks` to move completed canonical tasks into their archive section without erasing history.

## `context.md`

Context is the handoff surface for decisions that code and API tables cannot explain.

```markdown
---
spec: auth.spec.md
---

## Key Decisions

- Bulk revocation advances a per-user session generation instead of scanning tokens.

## Files to Read First

- `src/auth/session_store.rs`
- `tests/session_revocation.rs`

## Current Status

- The generation design is active; multi-region propagation remains roadmap work.

## Notes

- Preserve fail-closed behavior when the session store is unavailable.
```

Update context when an architectural decision, constraint, key file, or current implementation status changes. Avoid duplicating requirements or API rows.

## `testing.md`

Testing connects requirements and contracts to evidence.

```markdown
---
spec: auth.spec.md
---

## Automated Coverage

| Requirement | Test | Evidence |
|---|---|---|
| `REQ-auth-004` | `revokes_all_active_sessions` | Unit and integration suites |

## Manual QA

- Confirm active devices are signed out after bulk revocation.

## Edge Cases

- No active sessions
- Concurrent refresh and revocation
- Session-store timeout
```

Every requirement introduced by an active semantic delta must appear in verification evidence before acceptance. Record what is actually tested; do not turn intended future coverage into a passing claim.

## Optional `design.md`

Enable design companions with:

```toml
[companions]
design = true
```

Use `design.md` for layout, component hierarchy, interaction states, tokens, accessibility decisions, and assets. Behavioral requirements still belong in `requirements.md`; implementation work still belongs in `tasks.md`.

## Relationship to change workspaces

Canonical companions describe the current module. `.specsync/changes/CHG-*` describes a proposed delivery:

| Canonical companion | Change-workspace counterpart |
|---|---|
| `requirements.md` | Requirement additions, modifications, or removals in semantic deltas |
| `tasks.md` | Finite implementation tasks required before verification |
| `context.md` | Change-specific constraints and background |
| `testing.md` | Planned and recorded evidence for affected requirement IDs |

At acceptance, SpecSync applies semantic deltas atomically to canonical truth. The change workspace remains active until delivery is integrated, then archives as immutable history.

Continue with the [spec format](spec-format.md), [workflow guide](workflow.md), or [quick start](quickstart.md).
