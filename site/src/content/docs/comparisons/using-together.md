---
title: "Using SpecSync with Spec Kit or OpenSpec"
description: "Map equivalent artifacts and assign one clear owner for intent, implementation context, and enforceable contracts."
section: "Comparisons"
order: 3
---

Different filenames do not mean missing capability. The tools package similar knowledge around different units: SpecSync centers durable modules plus verified changes, Spec Kit centers feature execution, and OpenSpec centers proposals plus canonical behavior.

## Artifact equivalence

| Knowledge or gate | SpecSync | Spec Kit | OpenSpec |
|---|---|---|---|
| Project principles | Policy `principles_file` / repository instructions | `.specify/memory/constitution.md` | Project configuration and agent instructions |
| Product intent and acceptance criteria | Module `requirements.md`; change `requirements.md` | Feature `spec.md` | `proposal.md` plus delta requirements/scenarios |
| Current technical/module contract | `*.spec.md` | Feature `spec.md` under a chosen persistence model | `openspec/specs/<domain>/spec.md` |
| Proposed contract change | `deltas/<module>.md` | No direct core semantic-delta artifact | `changes/<name>/specs/<domain>/spec.md` |
| Architecture and rationale | Durable `context.md`; optional change `design.md` | `plan.md`, `research.md`, `data-model.md`, `contracts/` | `design.md` and `proposal.md` |
| Work breakdown | `tasks.md` | `tasks.md` | `tasks.md` |
| Quality/evidence plan | `testing.md`, requirement evidence, verification record | `checklists/`, analyze/converge results, workflow logs | validate/verify results and completed tasks |
| Definition review | Mandatory digest-bound approval | Optional workflow gate or team review | Proposal review convention |
| Closing review | Mandatory closing approval bound to verification | Optional workflow gate/team review | Archive confirmation/workflow |
| History | Accepted workspace then immutable archive | Feature folders under the team's persistence model | `changes/archive/` |

The rows are equivalent by **purpose**, not identical guarantees. For example, Spec Kit's `analyze` and OpenSpec's verification can reason across artifacts through an agent, while SpecSync's `verification.json` and source validators are deterministic records consumed by CI.

## Pattern 1: Spec Kit for discovery, SpecSync for durable enforcement

1. Use Spec Kit to clarify the feature, explore architecture, and produce the plan/tasks.
2. Translate stable product requirements into module-scoped SpecSync requirement IDs.
3. Put enforceable exports, dependencies, invariants, and schema expectations in `*.spec.md`.
4. Implement with the same coding agent.
5. Require `specsync check --strict` and the SpecSync acceptance gate in CI.

This keeps Spec Kit's rich planning ecosystem without making generated feature artifacts the only long-term source of truth.

## Pattern 2: Adopt an OpenSpec repository

```bash
specsync change adopt --dry-run
specsync change adopt
```

The import preserves canonical and active OpenSpec material for migration. After adoption, choose one owner for semantic deltas. Maintaining the same requirement independently in both active change systems invites conflicting canonical merges.

## Pattern 3: Split ownership by boundary

- Use Spec Kit or OpenSpec for product/feature exploration.
- Use SpecSync for library modules, public exports, database schemas, cross-module dependencies, and merge enforcement.
- Link the feature artifact to stable SpecSync requirement IDs and change IDs.

The rule is simple: overlap in context is useful; overlap in canonical ownership must be explicit.
