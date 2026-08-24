---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: docs
---

# Docs

No user-facing command, flag or output format changes, so no `site/` page changes.

The operator-visible surface is one new refusal from `specsync change check` and
`specsync change accept`:

```
semantic delta for `auth` changed after approval; the approved wording is what rewrites the
canonical spec, so re-run `specsync change approve <id>` to approve the current delta bodies
(or restore them)
```

The message names the module and the remedy, because both legitimate ways into this state — you
edited the delta on purpose, or something else did — are resolved by looking at that one file and
either re-approving it or putting it back.

The durable record of the behaviour is the canonical spec: invariant 3 (corrected), invariant 36
(new) and `REQ-change-089`, plus the paragraph in `specs/change/context.md` that previously
described this as an open hole.
