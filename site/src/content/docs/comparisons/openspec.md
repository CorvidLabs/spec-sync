---
title: "SpecSync vs OpenSpec"
description: "Compare two brownfield SDD lifecycles with semantic deltas, canonical truth, and different enforcement boundaries."
section: "Comparisons"
order: 2
---

SpecSync and OpenSpec share a useful brownfield model: current truth is separate from proposed changes, deltas evolve that truth, and completed work can be archived. The biggest difference is what the core can prove without asking an agent to interpret the repository.

## Shared shape

Both tools provide:

- canonical specifications separate from active changes;
- proposal or interview-driven change workspaces;
- ADDED, MODIFIED, and REMOVED requirement deltas;
- design and task artifacts;
- validation before archival; and
- an archived history of completed changes.

## Enforcement boundary

| Question | SpecSync 5.1 | OpenSpec core |
|---|---|---|
| Canonical current behavior | Module `requirements.md` + `*.spec.md` | `openspec/specs/<domain>/spec.md` |
| Proposed intent | `change.md`, interview answers, companions | `proposal.md` |
| Semantic changes | Module-scoped deltas | Change-local delta specs |
| Technical design | Optional `design.md`, durable `context.md` | `design.md` |
| Work tracking | `tasks.md` | `tasks.md` |
| Definition approval | Mandatory digest-bound human gate | Review/agent workflow convention |
| Verification | Configured commands plus requirement evidence | Validation and agent-led verification |
| Real export/schema parsing | Built into the deterministic core | Not built into the core artifact workflow |
| Canonical merge | Atomic during acceptance | Applied during sync/archive workflow |
| Post-change history | Accepted workspace, then immutable archive | Archived change folder |

OpenSpec is lighter when the team wants a proposal-oriented agent workflow and semantic requirements without language-specific enforcement. Its default core profile keeps a small propose/apply/sync/archive surface, while verification, onboarding, bulk archival, and stepwise artifact control are available through expanded workflows. SpecSync carries more machinery because it also acts as a CI contract checker: a documented export that disappeared, an undocumented public export, a missing source file, or database-schema drift becomes a deterministic finding.

## Choosing

Choose OpenSpec when a lightweight, agent-friendly proposal/delta/archive workflow is the main goal. Choose SpecSync when that lifecycle must also bind approvals and tests to current inputs and parse the implementation itself. If adopting an existing OpenSpec project, `specsync change adopt --dry-run` previews the import before any files change.

Official references: [OpenSpec 1.6.0](https://github.com/Fission-AI/OpenSpec/releases/tag/v1.6.0), [getting started and artifact model](https://github.com/Fission-AI/OpenSpec/blob/main/docs/getting-started.md), [commands](https://github.com/Fission-AI/OpenSpec/blob/main/docs/commands.md), and [workflows](https://github.com/Fission-AI/OpenSpec/blob/main/docs/workflows.md). Verified 2026-07-14.
