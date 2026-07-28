---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: testing
---

# Testing

## Characterization

- Build a lifecycle change with at least 393 signed acceptance entries.
- Complete accepted → stale → reopen → verify → accept for A/B/A manifest states.
- Before the fix, assert each reopening embeds another full manifest in `approvals.json`.

## Targeted Regression Coverage

- `REQ-change-043`: compact object reuse, bounded A/B/A growth, strict object validation, mixed
  v1/v2 migration, protected exact-v1-prefix compatibility, per-v2 prior-root authentication,
  reacceptance, and archive regressions provide direct implementation evidence.
- New reopening events serialize without an embedded acceptance manifest.
- A/B/A history creates exactly two immutable objects and reuses A byte-for-byte.
- Ledger-size growth for another identical reopen is bounded independently of manifest entry count.
- Schema-v1 embedded events load, check, reopen again, reaccept, and archive.
- Schema-v2 events hydrate to the same prior `VerificationRecord` observed by lifecycle callers.
- Schema history permits v1-to-v2 migration but rejects a v2-to-v1 downgrade, and any v2 prefix
  permanently requires the complete audit binding.
- Missing, symlinked, non-file, oversized, malformed, unknown-field, path/digest-mismatched, and
  verification-inconsistent objects fail closed before mutation or trust.
- Existing objects with identical content are reused; conflicting bytes at the same digest fail.
- Failed ledger publication never creates a trusted dangling reference.
- Active-to-dated-archive moves preserve object resolution and authenticated history.
- `migrate 5.0` repairs only eligible legacy digest fields, is idempotent, and leaves compact
  records byte-identical.
- Staged lifecycle evidence with extra unstaged edits fails before a referenced workspace object
  can authenticate it.
- Staged deletion of a trusted lifecycle file fails while genuinely new untracked lifecycle files
  remain compatible.
- Historical candidates share commit and byte budgets across root, transition, recording, and
  archive scans; rejected candidates are charged before every early exit, and oversized individual
  lifecycle files fail before content parsing.
- Terminal workspace integration is independent of alternate indexes and assume-unchanged flags;
  terminal remote-history fallback is capped and uses the shared historical cache.
- Repeated references load a digest once while event and expanded-hydration limits bound memory.
- Migration rejects same-inode/same-length races and uses retained state, verification, ledger, and
  object capabilities after ambient workspace replacement.
- A final-instruction migration edit is detected from displaced bytes and rolled back atomically.
- Sparse oversized approval ledgers fail metadata preflight, while excess approval/reopening arrays
  fail streaming preflight before typed allocation.
- Final-window lifecycle-leaf and workspace-ancestor swaps, plus transaction-recovery symlinks,
  cannot mutate outside files and leave no temporary publication residue.
- Git repositories ignore force-tracked worktree journals; recovery rejects `.git` and unrelated
  targets, rejects entry excess before reads, streams aggregate escaped-byte accounting before full
  serialization, survives governed-root renames, ignores ambient Git repository overrides, and
  durably records completion.
- A lock leaf replaced after kernel locking triggers identity-based retry and leaves the named lock
  mutually exclusive.
- Migration event limits run before generic JSON/range allocation, and a final-window workspace
  replacement restores the detached original bytes without touching the replacement workspace.
- Archive moves atomically reject a racing destination, verify the moved identity, roll back
  post-move failures, synchronize both retained parents, and recover exact accepted state after
  simulated process loss immediately after the move.
- Compact manifest reads across multiple historical candidates are charged through the same shared
  history budget and exact commit/path cache as state, verification, and approval blobs.
- Archive finalization and forced rollback preserve exact accepted source bytes through retained
  capability-relative moves and publications.
- Every reopening, not only the first, requires its exact protected accepted root; stripping a
  later object reference and recomputing nested/top-level unsigned digests fails terminal
  authentication and the next reopen before mutation.
- A second reopen before the preceding refreshed acceptance is protected leaves state and approvals
  byte-identical. Reacceptance verification binds the complete newest audit prefix, and post-verify
  actor/reason/transition/digest tampering fails before acceptance writes.
- Transaction and archive recovery restore only recognized original/replacement/staged/final byte
  states and preserve unknown post-journal edits with their journals.
- Migration revalidates retained state and verification inputs in the final exchange window.
- Correction scans use bounded sanitized Git reads and reject rollback/divergence, while unborn
  repositories pass only with an empty correction ledger.
- Legacy whole-tree reconstruction rejects excessive entries or aggregate blob bytes before
  materialization, deduplicates repeated tree identities, and never invokes checkout hooks or
  content filters.
- Git and non-Git project locks remain authoritative when a governed worktree lock leaf is
  replaced; Windows never flushes an unsupported read-only directory handle.
- Protected remote-default discovery ignores hostile ambient repository overrides and rejects a
  symbolic target outside `refs/remotes/origin/`, including terminal accepted-evidence
  authentication.
- Terminal accepted-workspace integration rejects forged bytes hidden by alternate indexes or
  assume-unchanged flags and compares unexpected paths, modes, and bytes directly with the trusted
  remote tree.
- Terminal anchors ignore local Git replacement refs, and a final-window lifecycle artifact swap
  cannot separate parsed evidence from the retained inventory being compared.
- Terminal remote-record fallback charges every candidate through the shared 4,096-commit history
  budget/cache and fails closed on the first excess candidate.
- Windows legacy reconstruction preserves signed `100755` regular-file and `120000` symlink modes
  without relying on host filesystem classification, and manifestless governed `160000` Gitlinks
  remain in the signed inventory.
- Non-Git locking never trusts a predictable namespace pre-created beneath a shared temporary
  directory; differing XDG/HOME/TEMP environments still contend on one OS-account-derived
  authority, including on Windows.
- Repeated audited reacceptances succeed through sequential protected squash merges and reject a
  second reopen until the immediately preceding acceptance is protected.
- A protected closing-valid accepted root containing an exact two-event schema-v1 ledger preserves
  a squash-discarded legacy intermediate root; changing one event byte removes that compatibility
  and fails closed without a per-event v1 fallback.

## Broader Gates

- Targeted Rust unit and CLI integration tests.
- Full unit, integration, release-build, docs, audit, Linux/macOS/Windows CI matrix.
- `specsync change check`, `specsync check --strict`, 100% spec coverage, and score at least 80.
- `fledge lanes run verify` and `fledge trust verify`.
- Augur must not block; Attest provenance is recorded only after verification passes.
- One independent reviewer checks every #466 acceptance row and another performs adversarial
  persistence, path, compatibility, and regression review.

## Executed Evidence

- `fledge run test -- reopen`: all 26 reopening unit tests and all three matching integration tests
  pass after upgrading every multi-reopen fixture to preserve each protected committed acceptance
  root.
- Focused regressions pass for staged-ledger/object atomicity, pre-mutation unanchored rejection,
  remote-only root authentication, exact later-event tamper rejection, forged local prefix
  rejection, A/B/A object reuse, and bounded ledger growth.
- Focused migration regressions pass for byte-identical dry runs, exact-byte preservation outside
  inserted fields, mixed v1/v2 idempotence, no-follow workspace/leaf rejection, and atomic repair.
- `fledge run test -- manifest`: all 87 matching unit tests and all eight matching integration tests
  pass after fixture expectations were tightened for protected roots and index-bound objects.
- Focused regressions pass for staged/worktree divergence, first-excess-byte and event rejection,
  one-read manifest hydration with bounded expansion, retained migration inputs, and
  same-inode/same-length migration races.
- Focused regressions pass for staged deletion, sparse oversized-ledger metadata preflight,
  streaming approval/reopening event preflight, shared terminal history budgets, final-window
  migration rollback, lifecycle leaf/ancestor swap confinement, transaction-recovery symlink
  refusal, and retained archive rollback.
- The first final-tree full run passed 2,401 tests (2,070 unit and 331 integration). After the
  independent-review hardening, focused tests pass for migration races/preflight, project-lock
  replacement, no-replace archive publication, tracked-journal refusal, target-constrained
  recovery, and oversized escaped journals.
- `x86_64-pc-windows-gnu` cross-check with warnings denied passes after the handle-relative Windows
  replacement and cross-platform directory-sync changes.
- `fledge run test -- trusted_lifecycle_git_entries_are_size_and_mode_bounded`: passed.
- `fledge run lint` passes after the latest filesystem, migration, journal, lock, archive, staging,
  and budget hardening.
- Focused regressions pass for unknown-edit-preserving transaction and archive recovery,
  final-window migration input edits, bounded correction history, and bounded legacy-tree
  reconstruction.
- Focused reviewer regressions pass for transaction-journal clear CAS, post-publication migration
  rollback, private non-Git lock directories, Windows publication flush coverage, sanitized nested
  correction prefixes, aggregate/deduplicated legacy reconstruction, and hook/filter-free raw blob
  materialization.
- Final persistence-review characterizations pass: nine project-lock tests, six legacy
  reconstruction tests, protected-reference namespace rejection, and ambient
  `GIT_DIR`/`GIT_WORK_TREE` isolation.
- `x86_64-pc-windows-gnu` cross-check with warnings denied passes on the latest tree, including the
  host-independent `100755`/`120000` reconstruction evidence.
- The final focused recursive semantic-successor regression passes after reloading the archived
  successor from retained state; the test still proves recursive successor coverage for the
  original accepted change while preserving exact terminal-snapshot binding.
- The repository-wide lifecycle scan authenticates all 71 pre-existing terminal histories after
  applying exact protected schema-v1-prefix compatibility; its only remaining findings are the
  current CHG-0068 approval/evidence gates.
- Final `fledge run test` passes all 2,116 unit tests and all 331 integration tests on the reviewed
  tree.
- Both independent review tracks completed with no unresolved high- or medium-severity findings
  after the final replacement-ref, retained-snapshot, bounded-history, lock-authority, and legacy
  reconstruction hardening.

Formatting, audit, strict spec, score, repository verification, trust, and provenance gates remain
to be rerun on the final tree.
