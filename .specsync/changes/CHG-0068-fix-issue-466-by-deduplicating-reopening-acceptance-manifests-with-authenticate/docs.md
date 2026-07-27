---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: docs
---

# Docs

Update the canonical `change` spec and companions to document:

- compact schema-v2 reopening events and immutable content-addressed manifest objects;
- backward-compatible reads of schema-v1 embedded events;
- fail-closed object hydration and archive behavior;
- bounded serialized evidence growth and A/B/A object reuse;
- the fact that `migrate 5.0` does not rewrite already-valid compact or legacy history.

Add the accepted CHG-0068 changelog row through the lifecycle. User-facing command syntax is
unchanged, so no new CLI guide page is required. Release notes should identify the on-disk
optimization and compatibility guarantee.
