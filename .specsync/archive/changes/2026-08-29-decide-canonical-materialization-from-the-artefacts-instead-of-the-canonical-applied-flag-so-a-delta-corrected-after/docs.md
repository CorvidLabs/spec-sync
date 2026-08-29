---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: docs
---

# Docs

## What an operator sees differently

**Correcting a delta after review now works.** Previously the sequence *correct the delta →
`change approve` → `change check`* recorded a new approval, exited 0, and left the canonical spec
carrying the superseded wording. It now materializes the correction. No new command, no new flag —
the documented loop simply does what it says.

**Two refusal messages name a second step.** `ensure_approved_delta_bodies_unchanged` used to end
at "re-run `specsync change approve <id>`", which was the action that walked the author into the
silent skip. Both of its refusals now say to run `specsync change check <id>` afterwards:

```
semantic delta for `change` changed after approval; the approved wording is what rewrites the
canonical spec, so re-run `specsync change approve <id>` to approve the current delta bodies and
then `specsync change check <id>` to materialize them into the canonical spec (or restore the
approved delta bodies)
```

**The CI refusal names the modules.** When `change check` runs in CI with canonical specs that do
not carry the approved deltas, the message is now:

```
the canonical specs do not carry the approved semantic deltas (`change`); run `specsync change
check` locally and commit the result before CI
```

The previous wording — "approved canonical deltas are not materialized" — was true only of a
never-materialized change, and this refusal is now also reachable for a change whose materialization
is behind its delta.

## What does not change

- A `change check` over a change whose canonical outputs are all current still writes nothing. It
  does not rewrite the spec, does not re-bump the version, and does not append a second Change Log
  row. One change bumps one module's version exactly once.
- No file format, schema version, or persisted field changed. Existing change workspaces and every
  archived ledger read exactly as before.
- No documentation page in `site/` describes the short-circuit, so none needed editing. The
  behaviour is stated where it is enforced: Invariant 41 and `REQ-change-092` in `specs/change/`.
