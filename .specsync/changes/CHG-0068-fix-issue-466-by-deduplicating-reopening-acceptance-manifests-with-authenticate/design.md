---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: design
---

# Design

## Persisted Representation

New reopening events use schema version 2. Their prior verification keeps every existing field
except the large `acceptance_manifest` payload and carries a versioned manifest reference when the
prior verification signed a manifest. The reference contains only the validated manifest digest;
the object path is derived internally and is never accepted from persisted input:

`.specsync/changes/<change>/evidence/acceptance-manifests/<digest>.json`

Archived workspaces use the same path relative to their dated change directory. The object is the
deterministic JSON representation of `AcceptanceManifestV1`. Its filename digest must be lowercase
SHA-256 syntax and must equal both the validated manifest digest and the prior verification's
`acceptance_input_digest`.

## Compatibility Boundary

The public in-memory `ReopenRecord` remains hydrated with a complete `prior_verification`.
Persistence uses private versioned disk records:

- schema v1 reads the existing embedded verification unchanged;
- schema v2 reads the compact verification plus its manifest reference and hydrates the manifest
  before returning any trusted ledger;
- new writes emit schema v2 and never rewrite existing schema-v1 events merely because the ledger
  receives another approval.

Histories may transition from schema v1 to v2 once and never downgrade. Once any v2 event exists,
all later verification and closing evidence must carry the complete reopening-prefix binding even
if a hostile ledger attempts to append a schema-v1-shaped event.

Approval mutations share one location-aware loader/writer so ordinary approvals, portable
definition pairs, reopen, reaccept, migration, check, and archive cannot bypass hydration or
accidentally expand compact events.

Schema dispatch is explicit rather than an untagged fallback. Compact prior-verification parsing
tracks whether `acceptance_manifest` appeared at all, so both an embedded object and an explicit
`null` are rejected. Approval, verification, and reopening records reject unknown fields before
they can participate in exact-history comparisons.

## Immutable Object Rules

- Derive object paths from a validated digest; persisted records cannot supply filesystem paths.
- Reject traversal, symlink components, non-files, oversized payloads, unknown fields, malformed
  manifests, filename/content digest disagreement, and verification/reference disagreement.
- If an object already exists, require byte-identical deterministic content and reuse it.
- Write a new object with the repository's atomic no-follow transaction before publishing the
  referencing ledger. A failed ledger write may leave an unreferenced immutable object but cannot
  leave a trusted dangling reference.
- Resolve ancestors and leaves relative to retained directory capabilities. Leaf reads use
  nonblocking no-follow opens before descriptor validation, so a regular-file-to-FIFO swap fails
  instead of blocking.
- Synchronize object bytes and immutable publication before returning, then verify deterministic
  bytes and the retained directory identity again.
- Never mutate or delete an existing manifest object as part of reopen, reaccept, or migration.

## Historical Authentication

Each reopening's nested superseded approval must equal a unique, monotonically ordered top-level
approval event. Before any reopen writes anything, every schema-v2 event must match an accepted
snapshot reachable from the protected remote-default reference. The match binds the exact prior
verification, approval prefix through that event's superseded approval, and all preceding reopening
records. A contiguous schema-v1 prefix remains compatible only when one protected closing-valid
accepted snapshot contains the exact complete v1 approval and reopening prefix. Because a prior
root does not authenticate the current v1 event's unsigned audit
fields, there is no per-event v1 fallback: an absent or byte-different exact prefix root fails
closed. The compatibility root cannot cover a schema-v2 event or a non-prefix v1 event.
Consequently every compact refreshed acceptance must enter protected history before another reopen
can audit it, while immutable pre-v2 histories remain independently verifiable after their
intermediate commits were squash-discarded.

Protected-reference discovery runs through the same target-rooted, environment-sanitized Git
runner as every subsequent history read. The symbolic target must remain beneath
`refs/remotes/origin/`; ambient `GIT_DIR`, `GIT_WORK_TREE`, and related repository overrides can
neither choose the branch name nor split discovery from authentication. This applies to terminal
accepted-evidence authentication as well as reopening-root discovery.

Terminal squash-integration fallback compares a bounded retained-handle worktree inventory
directly with the trusted remote tree, including mode, kind, bytes, and unexpected paths. It does
not consult the caller's index, `GIT_INDEX_FILE`, or skip/assume flags. Remote accepted-record
history enumeration shares the normal commit/blob budget and cached historical state loader,
rejecting the first candidate beyond the bound.
Every terminal-anchor Git command disables replacement objects. The retained inventory is also
bound to the exact state, verification, and approval artifact bytes parsed earlier in the same
authentication attempt, so an atomic artifact swap cannot combine one snapshot's evidence with
another snapshot's remote-tree comparison.

Schema-v2 verification records carry a versioned framed digest over every reopening event field,
its superseded approval, and the compact prior verification. Verification recomputes this digest
from the exact hydrated ledger, closing approval includes it, and subsequent loads require it to
match. Schema-v1 evidence remains byte-compatible and does not gain a synthetic binding.

Freshly accepted working-tree evidence is eligible for pre-commit validation only when its closing
digest and verification are current. When `approvals.json` comes from the index, every compact
manifest reference is captured from that same index generation and must be a staged mode-`100644`
regular file. State, verification, and approvals must also be byte-identical between index and
worktree whenever the index differs from `HEAD`; extra unstaged lifecycle edits fail closed.
An absent index entry is accepted only for a genuinely new lifecycle file; if `HEAD` contains the
path, the staged deletion fails closed. Historical state, verification, and approval reads are
individually bounded and charged immediately under one shared terminal-scan commit/byte budget,
including rejected candidates and candidates reached through multiple refs. Referenced Git
manifest objects are individually size-bounded and mode-checked.

Hydration caches each distinct manifest digest for one ledger load, so aliases cannot trigger
repeated object reads or parsing. Separate unique-byte, expanded-byte, reopening-event, and
approval-event limits bound both I/O and the hydrated in-memory ledger even when a compact history
repeats one object reference thousands of times. Worktree ledger size is rejected from metadata
before allocation, and a streaming JSON visitor rejects excess event arrays before deserializing
their elements into typed vectors.

## Confined Lifecycle Publication

Lifecycle mutations retain the canonical project root and traverse each parent component with
no-follow directory opens. Existing regular files are published with an atomic exchange; the
displaced identity and exact bytes are verified before deletion, and mismatch rolls the exchange
back. New files use a synchronized create-new temporary plus a hard link, so a raced destination
cannot be overwritten. Successful publication and deletion synchronize the retained parent.

Git worktrees store project-keyed crash journals beneath their per-worktree Git metadata directory,
outside tracked content. The key uses the stable repository-relative governed-project prefix, and
metadata discovery uses the rooted, environment-sanitized, fail-closed Git runner. Journals are
explicitly versioned, entry-count checked before backup reads, byte-accounted through streaming
JSON serialization, bounded before publication, and may restore only direct lifecycle-workspace
files or canonical spec/requirements targets; `.git` and unrelated project files are rejected.
Every current-schema entry records original and intended replacement bytes. Publication compares
the retained target with the original; recovery restores only the exact replacement and rejects an
unknown third state without overwriting it. Legacy journals clear only when every target remains
the recorded original. All journal deletion compares the exact previously read or published bytes.
Non-Git fallback journals enforce the same target policy.

Git project locks reside under Git metadata; non-Git locks use a canonical-root-keyed location
inside an authenticated, already user-owned private runtime directory outside the governed
worktree. A predictable namespace directly beneath a shared temporary directory is never trusted.
The authoritative location is derived from the operating-system account rather than caller-varying
`XDG_RUNTIME_DIR`, `HOME`, or temporary-directory variables, so processes with different
environments still contend on one lock. Unix validates no-follow directory type, effective-user
ownership, and private permissions. Windows uses a user-private account location, rejects reparse
directories and parent replacement, and never trusts a predictable shared `%TEMP%` namespace. The
named lock leaf is identity-checked after acquiring its kernel lock, with bounded retry on
replacement. Directory synchronization remains mandatory on Unix. Windows reopens and flushes
every newly published hard-link or replacement name before dependent mutations because flushing a
read-only directory handle is not supported there.

Archive moves use kernel no-replace directory renames between retained active and archive parents.
The moved identity is verified, every post-move mismatch is rolled back through the same retained
handles, and both parents are synchronized. Final state, markdown, and rollback writes use the same
exchange protocol. Symlink/reparse leaves, ancestor replacement, racing destinations, non-files,
and parent identity changes fail closed without touching attacker-selected outside paths.
A separate durable archive-move journal is published before staging the accepted snapshot. Its
prepared, moved, and finalized phases bind original, staged, and final state/markdown bytes. On the
next lock, recovery recognizes only exact phase/topology combinations, safely advances or rolls
back partial publication, and preserves any unknown workspace edit together with the journal. It
is cleared by exact compare-and-swap only after successful archived-evidence validation.

## Legacy Migration

`migrate 5.0` enumerates active and archived workspaces without following symlinks, retains each
workspace as a directory capability, and opens state, verification, and approval leaves with
nonblocking no-follow semantics. The repair scanner identifies exact reopening-object byte ranges
and inserts only absent legacy digest fields. Streaming event preflight applies the normal approval
and reopening limits before generic JSON or range-vector allocation. It validates the patched ledger before an
identity-checked, synchronized atomic replacement, preserving every unrelated byte. Non-dry-run
migration holds the project lifecycle lock. Live digest derivation, compact-object hydration, and
repaired-ledger validation consume only retained snapshot capabilities. The target's exact bytes
are checked immediately before an atomic exchange, and state plus verification bytes/absence are
revalidated as part of the same framed migration-input binding, then the displaced bytes are
verified. Any edit or workspace binding change in the final instruction window triggers a
synchronized atomic rollback, so an in-place edit cannot be overwritten merely because inode
identity and length stayed unchanged. The retained state/verification binding is checked again
after approval publication and before the displaced ledger is deleted; mismatch rolls the exchange
back.

Legacy acceptance reconstruction preflights the entire commit tree before adding a temporary
materialization. Only portable paths and expected Git object modes/types are accepted. Accepted
anchors are deduplicated by tree object identity, and tree-entry count plus aggregate unique blob
bytes are charged across the whole invocation. Files are populated from bounded Git object reads
without checkout, hooks, or content filters. Reconstruction carries each entry's Git mode and kind
as explicit evidence instead of deriving them from host filesystem metadata, preserving `100755`
and `120000` authentication on Windows. Governed reconstruction-only keys, including `160000`
Gitlinks that filesystem discovery cannot represent as files, are merged into the deterministic
path inventory before digest and manifest generation. All repository-prefix and tree discovery
uses the sanitized bounded Git runner with ambient repository/index overrides removed.

## Growth Bound

An A/B/A history creates at most objects A and B. Each reopening appends only bounded event and
reference metadata. Reusing A does not rewrite it, and object storage grows once per distinct
validated manifest rather than once per reopening.

## Non-goals

- No bulk conversion of historical schema-v1 reopening events.
- No compaction of the separate correction ledger.
- No change to acceptance-manifest semantics, owner resolution, freshness, or closing digests.
