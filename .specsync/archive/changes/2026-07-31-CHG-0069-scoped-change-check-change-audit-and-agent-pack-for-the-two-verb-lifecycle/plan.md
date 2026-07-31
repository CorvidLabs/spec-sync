---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: plan
---

# Plan

1. Detach `check_project` from CLI `change check`; print scoped success footer.
2. Add `change audit` calling active-only `audit_project`.
3. Extend agents install with check/audit commands; update skill and hooks.
4. Hard-cut Agents.md and site docs.
5. Update `change` module spec Public API for `audit_project`.
6. Dogfood: audit ~seconds; check dominated by verification commands only.
