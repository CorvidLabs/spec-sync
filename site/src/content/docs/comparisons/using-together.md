---
title: "Using SpecSync with Spec Kit, OpenSpec, or BMAD"
description: "Map planning artifacts to one clear owner for intent, implementation context, and enforceable contracts."
section: "Comparisons"
order: 4
---

Different filenames do not mean missing capability. The tools package related knowledge around different units: SpecSync centers durable modules plus verified changes, Spec Kit centers feature execution, OpenSpec centers proposals plus canonical behavior, and BMAD centers progressive product-development context.

## Artifact equivalence

| Knowledge or gate | SpecSync | Spec Kit | OpenSpec | BMAD |
|---|---|---|---|---|
| Project principles | Policy / repository instructions | Constitution | Project config/instructions | Project context |
| Product intent | Requirements/change artifacts | Feature spec | Proposal plus deltas | Brief, PRD, or lean SPEC |
| Technical contract | `*.spec.md` | Feature artifacts | Canonical domain spec | Architecture/story context |
| Architecture | Context/design | Plan/research/contracts | Design/proposal | Architecture workflow |
| Work breakdown | Tasks | Tasks | Tasks | Epics and stories |
| Quality/evidence | Testing plus verification record | Checklists/analyze/workflow logs | Validate/verify results | Readiness, review, optional TEA |
| Definition review | Mandatory digest-bound gate | Optional workflow gate | Review convention | Workflow checkpoint |
| Closing review | Mandatory evidence-bound gate | Optional workflow/team review | Archive workflow | Code/release review workflows |
| History | Accepted then immutable archive | Chosen persistence model | Change archive | Planning and implementation artifacts |

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

## Pattern 3: BMAD for product delivery, SpecSync for proof

1. Use BMAD's appropriate planning track to shape intent, architecture, and stories.
2. Translate durable technical boundaries into SpecSync requirements and module contracts.
3. Let BMAD agents implement and review while SpecSync checks the resulting repository.
4. Require SpecSync targeted verification, scoped review, and same-PR finalization before GitHub merge.

## Pattern 4: Split ownership by boundary

- Use Spec Kit, OpenSpec, or BMAD for product/feature exploration.
- Use SpecSync for library modules, public exports, database schemas, cross-module dependencies, and merge enforcement.
- Link the feature artifact to stable SpecSync requirement IDs and change IDs.

The rule is simple: overlap in context is useful; overlap in canonical ownership must be explicit.
