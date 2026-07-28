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

Implementation now uses a private, explicitly dispatched schema-v1/schema-v2 persistence boundary.
Compact records require the manifest-reference field to be present even when its value is `null`,
which prevents mixed legacy/compact shapes from being normalized accidentally. Approval,
verification, and reopening records reject unknown fields before exact event-prefix comparisons.
Schema evolution is monotonic (`v1*` followed by `v2*`): a v1 event can never follow a compact
event, and any compact event makes the complete-prefix verification binding mandatory forever.

Manifest objects are opened relative to retained directory capabilities with no-follow and
nonblocking leaf semantics. New objects are synchronized before and after immutable hard-link
publication; object directories are identity-checked before returning. Historical Git reads
preflight blob size, require canonical mode `100644`, and read by verified object identity.

Every reopening, including schema-v1 history, must match its exact prior verification, approval
prefix, and preceding reopening prefix in a protected remote-default accepted root. This prevents a
later event from being stripped or rewritten together with its unsigned nested approval while
retaining the initial anchor. A single refreshed acceptance remains squash-compatible, but another
reopen is blocked until the preceding refreshed acceptance is merged into protected history.
Reopen rejects any unanchored event before mutation. Compact reacceptance verification also signs a
framed digest of the complete reopening audit prefix; actor, reason, transition, and stale/current
digests therefore participate in closing evidence rather than remaining unauthenticated metadata.
Protected-reference discovery and all subsequent history reads now share the target-rooted,
environment-sanitized Git runner, preventing ambient repository overrides from choosing a branch
name in one repository and authenticating it in another, including during terminal accepted-state
authentication.
The terminal squash-integration fallback no longer trusts Git's mutable index view: it compares a
bounded retained-handle workspace inventory directly against the protected tree, so alternate
indexes and skip/assume flags cannot hide forged lifecycle bytes or unexpected files. Terminal
remote-history fallback uses the same bounded commit/blob cache as other trusted-history scans.
Replacement objects are disabled throughout that terminal command chain, and the compared
inventory must contain the exact state, verification, and approval bytes already parsed by the
authentication attempt.

Index-backed approval evidence binds every referenced manifest object into the same Git snapshot
and requires a staged `100644` object. A staged lifecycle file is rejected when its worktree copy
has additional unstaged changes or is staged for deletion while a trusted `HEAD` copy remains, so
authentication never combines index approvals with ambient manifest objects. Historical root and
terminal-transition searches share commit and aggregate-byte budgets and charge every state,
verification, and approval blob immediately, including candidates rejected before manifest
hydration. Referenced compact manifest blobs use that same invocation budget and exact commit/path
cache, so repeated terminal candidates cannot bypass aggregate history bounds. Repeated manifest
references are parsed once per digest and remain bounded by unique, expanded, and event-count
budgets. Approval byte limits are checked from no-follow metadata before allocation, and event
limits are enforced by a streaming preflight before typed vectors are built.

`migrate 5.0` now holds the project lifecycle lock, traverses retained no-follow workspace
capabilities, and derives state, verification, manifest objects, and repaired ledger validation
only from that retained snapshot. It patches only missing legacy fields and checks exact target
bytes immediately before synchronized atomic exchange. The displaced bytes are verified after the
exchange; a final-window edit or workspace-ancestor replacement rolls back atomically rather than
being overwritten. Approval/reopening event counts are streamed and bounded before the migration
allocates a generic JSON tree or byte-range vector.

All lifecycle state, artifact, approval, verification, transaction-recovery, archive-finalization,
sequence, policy, and lock publications now operate relative to retained project-directory
capabilities. Ancestors and leaves are opened without following links, existing files are replaced
with exchange-and-verify rollback, new files use create-new hard-link publication, and archive
directory moves use kernel no-replace operations relative to retained source and destination
parents, with identity verification, rollback, and durable parent synchronization. Successful file
publication and deletion also synchronize the retained parent directory on platforms that support
directory flushing; Windows skips the unsupported read-only directory-handle flush without
weakening file synchronization or atomic publication.

Crash-recovery journals no longer live in the tracked worktree for Git repositories. They are
keyed by the stable repository-relative project prefix beneath the current worktree's Git metadata
directory, carry an explicit schema, ignore ambient Git repository overrides, reject `.git` and
unrelated recovery targets, and are count- and byte-bounded while backups are read, before full
serialization or publication. Each entry binds both the snapshotted original and intended
replacement bytes. Recovery restores only an exact replacement, leaves an exact original alone,
and preserves any third-party edit with the journal for manual recovery. Legacy journals are
accepted only when every target still equals the recorded original. Non-Git fallback journals
apply the same target confinement. Git locks live beneath Git metadata, and non-Git locks beneath a
canonical-root-keyed authenticated user-owned private runtime directory with no-follow ownership
and permission checks; they never trust a predictable namespace directly beneath shared temporary
storage. Their authority is derived from the operating-system account rather than mutable process
environment, so different XDG/HOME/TEMP values cannot split mutual exclusion. Windows similarly
avoids shared `%TEMP%`, rejects reparse roots, and retains one account-private authority. Replacing
or pre-creating a shared worktree/runtime leaf therefore cannot split mutual exclusion.
The lifecycle lock is still re-opened and identity-checked by name after the kernel lock is
acquired; a raced replacement causes a bounded retry. Journal clearing is exact-byte
compare-and-swap, so a replacement recovery instruction cannot be deleted accidentally.
Deterministic leaf, workspace-ancestor, journal, lock-replacement, and archive-destination race
tests prove that attacker-controlled paths cannot redirect, broaden, or overwrite concurrent
lifecycle edits.

Archive staging, the directory move, and finalization are covered by a separate durable
project-keyed journal with prepared, moved, and finalized phases. Every phase binds the original,
staged, and final state/markdown bytes. Recovery advances or rolls back only recognized exact
states, preserves unknown post-journal edits, removes staging residue, and leaves archive retryable
whether a process dies before the move, immediately after it, or during final publication.
Successful finalization clears the exact journal durably only after archived evidence validates.

Correction-history authentication now uses the same sanitized, bounded Git runner and one shared
commit/blob budget for state, correction, approval, verification, and manifest reads. Historical
files must be mode `100644`, shallow boundaries are bounded, and an unborn repository is accepted
only when it has no correction events to authenticate. Legacy whole-tree reconstruction preflights
portable paths, object modes/types, entry count, and aggregate blob bytes before creating a
temporary materialization. Repository-prefix discovery ignores ambient Git overrides. Legacy
anchors are deduplicated by tree identity under one aggregate materialization budget, and objects
are materialized directly from bounded Git blobs so checkout hooks and smudge filters never run.
The recorded Git object mode and kind remain explicit reconstruction evidence, so Windows
authenticates legacy `100755` and `120000` entries without lossy host-filesystem reclassification.
Governed `160000` Gitlinks are inserted from sidecar evidence into the sorted path inventory even
though the temporary host filesystem represents them as directories.

Issue #467 previously prevented the required broad-scope regression from reaching the reopening
path because overlapping Git query batches duplicated stage-zero entries. CHG-0067 fixed and
archived that prerequisite on `main`; CHG-0068 starts from that merged commit.
