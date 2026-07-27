---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: context
---

# Context

Issue #466 measured a 393-entry acceptance manifest at roughly 103 KB minified. Because every
`ReopenRecord` currently embeds the complete prior `VerificationRecord`, 35 audited reopen events
repeated that manifest into a roughly 5.8 MB pretty-serialized `approvals.json`. The event metadata
outside those snapshots was only about 9 KB.

The acceptance manifest is already validated, deterministically serialized, and authenticated by
`acceptance_input_digest`. Lifecycle workspaces and their dated archive destinations are resolved
through location-aware, traversal-safe paths. Those properties allow the manifest payload to move
to an immutable content-addressed object without weakening the existing closing, stale-input, or
history checks.

The compact format applies only to new reopening events. Existing schema-v1 events retain their
embedded prior verification and remain readable without a bulk rewrite. In-memory lifecycle APIs
continue to expose a fully resolved prior verification so callers do not need to understand object
storage.

Issue #467 previously prevented the required broad-scope regression from reaching the reopening
path because overlapping Git query batches duplicated stage-zero entries. CHG-0067 fixed and
archived that prerequisite on `main`; CHG-0068 starts from that merged commit.
