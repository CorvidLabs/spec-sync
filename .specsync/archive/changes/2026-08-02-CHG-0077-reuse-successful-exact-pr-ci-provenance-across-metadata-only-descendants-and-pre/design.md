---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: design
---

# Design

Use one dependency-free helper to inspect a bounded first-parent list and authenticated GitHub check
metadata. Callers provide the expected check name, workflow path, repository, PR number, and current
metadata tip. The helper returns the nearest eligible successful ancestor or fails with diagnostics
that preserve rejected candidate reasons.

Trust metadata-child reuse and archive finalization call the same helper instead of duplicating
immediate-parent shell/API logic. Product tips continue running their current jobs. Metadata-only
descendants continue using exact diff classification and may skip the product matrix only after the
helper authenticates reusable ancestor evidence.

Trusted-policy verification sorts exact-SHA checks newest-first but accepts the first authenticated
success even if a newer matching publication was cancelled or failed. If no authenticated success
exists, it reports the rejected candidates and fails closed.

Historical metadata traversal recognizes both exact scoped-review pairs and workflow-v2 archive
moves. An archive edge is eligible only when one active change becomes its matching dated archive,
the archived state matches that change, and `finalization.json` binds the exact parent commit/tree.
Generic reusable checks must name a workflow job and bind that job's run, SHA, name, successful
conclusion, and check-run identity. Trusted-policy custom checks remain on their dedicated verifier:
an explicit workflow URL selects that run, while GitHub's canonical rewritten URL requires one unique
successful matching policy run and ignores later failed/cancelled publications.

To preserve a successful publication when GitHub reruns the same workflow run, the protected
publisher includes immutable run ID and run-attempt identity in the authenticated external binding.
The verifier resolves that exact attempt rather than trusting the mutable latest state of the run.

No CLI command, lifecycle state, approval count, artifact layout, or merge behavior changes. The
existing `change status` and `change finalize` path remains authoritative.
