---
change: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
artifact: plan
---

# Plan

## Approach

Keep `cmd_change` as the stable thin dispatcher and extract complete helpers into parent-private rendering, ship-status, archive-guidance, shipping, repository-status, owner-input and lifecycle-commit modules. Group existing tests by responsibility behind the existing test umbrella where useful. Preserve bodies, literal values, guards and explicit data flow; use the narrowest private visibility needed by siblings.

## Out of scope

No public API, lifecycle policy, security, output, exit code, Git behavior, network behavior or persistence format changes. Do not edit `src/change.rs`, `src/change_tests.rs`, `specs/change` or the active evidence/LFS work. Do not push, open a browser, alter authentication, invent human implementation review, accept or archive this change.

## Steps

1. Read AGENTS.md, the installed spec-sync skill, Human Intent and all cmd_change companions; record pristine focused tests and test inventory.
2. Complete this plan and the canonical internal-organization delta, then record only the already authorized scope approval with provenance.
3. Move complete production bodies into cohesive private modules and all existing tests/fixtures into topical files, targeting fewer than 1000 lines per source file.
4. Mechanically compare complete function bodies, test inventory, literals and cfg guards; inspect private visibility and unchanged call ordering.
5. Update canonical source mappings and companion context/testing/tasks, run formatting, focused tests, checks and clippy, and compare full-suite failures to the existing main baseline when useful.
6. Request independent agent code review, resolve mechanical defects within approved scope, run strict coverage and real `change check --commit`, then leave a clean local checkpoint and disclose unavailable trust/provenance gates.

## Risks

Moving rendering could accidentally introduce a live ledger reread after persistence or release a read scope too early. Moving readiness could substitute a local approximation for domain predicates. Splitting staging could alter NUL/rename parsing, literal pathspecs, exclusion warnings or sequence flooring. Moving shipping could reorder the gate and push or poll an unpushed archive. Retain every complete body unchanged and use regression plus mechanical comparison evidence at these seams.

## Constraints consulted

Read AGENTS.md, `.codex/skills/spec-sync/SKILL.md`, hi/agent.md, hi/change.md, hi/plan.md, hi/ship.md and specs/cmd_change/{cmd_change.spec.md,requirements.md,context.md,tasks.md,testing.md}. Preserve the lessons on transaction-bound mutation projections, best-effort text counts, fail-open briefing versus fail-closed trust, shared guidance, domain current-evidence predicates, literal scoped staging and archive/PR ordering. No configured principles file exists. User authorized whole-repository large-file refactoring, continuation and branch/commit work; this package records that scope only and does not claim human implementation review.
