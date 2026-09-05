---
spec: change.spec.md
---

# Tasks

- [x] Allow audited reopen of unreconstructible legacy acceptance with matching current inputs (#751).

- [x] Implement durable change state and adaptive artifacts
- [x] Implement deterministic interview and JSON projection
- [x] Implement semantic requirement/spec deltas
- [x] Implement digest-bound approvals and stale detection
- [x] Implement configured verification and atomic acceptance
- [x] Make `change check` in-process spec↔code sync rather than spawning `verification_commands`
- [x] Implement concurrent delta conflict detection
- [x] Implement adoption and external provenance import
- [x] Complete full release validation
- [x] Add protocol-clean quiet lifecycle checking for bounded PR reports
- [x] Add audited stale-accepted reopen, strict re-verification, and immutable evidence history
- [x] Preserve legacy definition digests when optional canonical-application state is false
- [x] Normalize compatible transitional definition evidence during explicit acceptance
- [x] Add repository-backed sequence claims and exact historical collision validation
- [x] Reject recursive verification while preserving append-only retry history
- [x] Allow only exact current canonical successors to govern stale predecessors
- [x] Resolve semantic delta targets through the committed registry
- [x] Resolve registry-backed paths during effective contract validation
- [x] Protect sequence claims and restrict collision baselines to immutable history
- [x] Support numeric change sequences wider than four digits
- [x] Reject recursion through every lifecycle command family
- [x] Reuse one project digest per canonical-successor scan
- [x] Detect recursive Cargo verification through safe explicit manifest selection
- [x] Cover exact canonical companion files without directory overreach
- [x] Preserve prose acceptance criteria with question-aware list parsing
- [x] Add audited append-only correction for supported accepted interview metadata
- [x] Harden trusted correction-history scans for unresolved remote refs and platform-valid quoted or Unicode Git paths
- [x] Add audited exact acceptance-owner corrections for reopened already-applied changes
- [x] Make stale accepted-change verification diagnostics actionable with named inputs and remediation
- [x] Trust squash-merged accepted evidence recorded in main history for archival
- [x] Repair adoption-era archived ledgers by assigning exact delivery ownership during legacy manifest reconstruction
- [x] Add transactional batch mode for exact acceptance-owner corrections
- [x] Add native 5.0→5.1 change-ledger migration with idempotent reopening digest backfill
- [x] Tolerate inert 5.0.1 registry stubs during canonical module path resolution
- [x] Deduplicate identical stage-zero entries from overlapping Git pathspec batches while rejecting conflicting mode or object pairs
- [x] Separate stable scope approval from volatile execution/evidence binding with plain-language expansion diagnostics
- [x] Freeze the CHG-0068 scope adoption without claiming unavailable cryptographic equivalence
- [x] Reject scope contraction, self-review, blocking review verdicts, and change-then-revert review reuse
- [x] Resume same-PR finalization after a crash that leaves the accepted workspace at its dated destination
- [x] Fail closed on missing adoption anchors, preserve append-only review attempts, recover partial
  terminal archive writes, share freshness limits, and authenticate squash-surviving v2 archives
- [x] Validate correction-ledger health inside locked answer, dependency, and supersession mutations
- [x] Return validated mutation snapshots so command rendering cannot fail after persistence
- [x] Capture normal/strict mutation summaries under lock and preserve documented production wrappers

- [x] Approve fails closed on ADDED of living REQ; draft next_action waits on complete artifacts (2026-08-01)
- [x] Bind semantic delta bodies to the definition approval and read an absent binding as unknown (2026-08-24)
- [x] Make the recorded delta binding monotone: the portable 5.0.1 pair records the wording it approves, and a later definition approval cannot withdraw a claim an earlier one made (2026-08-27)
- [x] Hash the delta binding over line-ending-canonical bytes so a CRLF checkout of an unedited delta stops failing the #711 gate, and fold nothing else (2026-08-27)
- [x] Name a held Cargo build-directory lock before the verification command that will wait on it, and run every verification child in its own process group so an interrupted check cannot outlive itself (2026-08-27)
- [x] Decide canonical materialization from the artefacts rather than the `canonical_applied` flag, so a delta corrected after review and re-approved is materialized again with its version bump and Change Log row, while a byte-identical re-approval still writes nothing (2026-08-29)
- [x] Let the archive preflight's closing package cover the legacy change it supersedes (forward the closing token into the successor walk; read a not-yet-committed successor's succession entry from the working tree its closing evidence signed), and name every refused successor with its reason in the stale-input diagnostic instead of "no successor covers it"; never offer `change reopen` of a workflow-v1 change beside a refused workflow-v2 successor; and make the active-only audit offer archived changes as successor candidates without evaluating them (2026-09-05)
