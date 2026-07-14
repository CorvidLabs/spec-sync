---
change: CHG-0025-address-all-unresolved-review-feedback-on-pr-366
artifact: context
---

# Context

PR #366 has nine unresolved automated review threads against the accepted CHG-0024 delivery. Eight expose fail-open or incomplete behavior in lifecycle integrity, registry resolution, static discovery, and scaffold validation; one identifies repeated repository hashing in canonical-successor evaluation.

CHG-0024 remains accepted with current evidence and cannot be silently reopened. This successor preserves that audit trail while governing the review-driven corrections across the `change`, `validator`, `config`, and `cli` modules.

The implementation will reuse the existing safe canonical-path resolver, extend the protected SDD path set to the committed sequence ledger, validate collision acknowledgements as exact sets containing only immutable accepted or archived records, recognize static files during zero-config directory detection, reject every generated design marker, apply recursion protection at CLI dispatch, and compute the project digest once per successor scan.
