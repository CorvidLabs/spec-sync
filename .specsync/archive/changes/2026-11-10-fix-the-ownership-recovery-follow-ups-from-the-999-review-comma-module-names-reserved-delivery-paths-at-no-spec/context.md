---
change: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
artifact: context
---

# Context

## Decisions

- The reserved-delivery exemption lives in
  `production_source_lacks_canonical_owner_with_evidence`, so approve-time validation and
  `correct-owner` discovery share it, matching the finalize-side exemption
  (`path_is_recognized_delivery_metadata` in the manifest builder).
- For comma-bearing module names the printed answer is omitted rather than escaped:
  `change answer` splits on commas and there is no escaping layer to fix, so the honest
  recovery is prose naming the modules plus the `files:` edit.
- The listing bound fails closed only where the index informs recovery: the ownership gate
  itself still works from declared specs and configured `owns`, which need no listing.

## Ruled out

- Changing `change answer` splitting or adding escaping: a wire-format change for a
  diagnostic's sake.
- Raising or removing the 10,000-entry bound: the bound protects every caller; the fix is
  to stop presenting truncated results as complete.
