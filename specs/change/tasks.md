---
spec: change.spec.md
---

# Tasks

- [x] Refuse to open a change when `.specsync/sdd.json` cannot be parsed (REQ-change-108).
- [x] Return from a bounded Git read when the direct child exits, even if a descendant still holds its output pipe (#854, #855, REQ-change-126, SPEED-1.a).
- [x] Show an opened change the lessons and principles it should honor, and require a selected plan to say how.
- [x] Report a change directory that holds files but no state.json as still open, and keep an empty husk skipped.
- [x] Group private Git capture options to fix PR #801's Clippy argument-count failure.
- [x] Name clean Git files by object id in the workspace digest, frame that id apart from file bytes, and accept `filter=lfs` only for the Git LFS driver and a pointer blob.
- [x] Replace child-written PID evidence with deterministic parent observation and stopped-child timeout coverage (#587).

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
- [x] Let a module own paths beyond its spec's `files:` through `[modules."<name>"] owns`, sign them under the module ahead of the reserved exact classes, judge a successor's eligibility for an exact-only predecessor entry by the module that owns the path now, read the claimants of a changed exact-only entry from the successors' declared obligations in the walk, and name the supersede alternative beside the audited reopen (2026-09-05)

- [x] Clarify scoped-review claims versus separately enforced authentication without changing runtime review rules.

- [x] Name the untracked paths a lifecycle commit may stage in the domain (`lifecycle_commit_scope`), excluding `affected_paths` prefixes and whole spec directories, and name the lock and transaction journal as never-committed runtime files.

- [x] Add authenticated same-delivery finalized policy coverage with exact signed-entry matching and nine regression cases (REQ-change-110).

- [x] Address PR #821 review with base archive identities across renames and local staged/untracked delivery coverage; add discriminating regressions.

- [x] Report an approval as not current in status, the handoff verdict and `change audit` when a semantic delta body it recorded was edited after approval, using the binding materialization and acceptance enforce, and name the edited modules in `summary.changed_approved_deltas` (REQ-change-111).
- [x] Run the effective-contract gate on the change being verified on every `change check`, the first one included, and record its refusal of a declared module as a failed attempt naming `specsync change check <id> (effective contract: …)` (REQ-change-112).
- [x] Leave files under an opt-in, validated `.specsync/sdd.json` `verification_ignored_paths` list out of the verification project-input digest and nowhere else, refuse an invalid list in every lifecycle verb and an entry overlapping a change's declared paths, and keep the empty-list digest byte-identical (REQ-change-114, REQ-change-115, issue #825).
- [ ] Follow-up, only if the human agrees: keep `change check --commit` from staging tracked edits under a listed prefix (issue #825 step G).
- [ ] Follow-up, only if the human agrees: relax the scoped-review descendant walk for listed prefixes, reading the list from the policy blob at the review's implementation commit and converting entries with `git_repo_relative_path`.
- [x] Refuse a `verification_ignored_paths` entry covering the configured `schema_dir`, a path-form spec `depends_on` entry or a spec's `Consumed By` file reference, all of which `change check` reads in-process (review finding on #830).
- [x] Scope owner: amend the REQ-change-115 acceptance criteria and invariant 24 to name `schema_dir`, path-form `depends_on` entries and `Consumed By` references; the refusal already enforces them under "overlaps no specs, sources, SDD policy inputs".
- [x] Write the lesson bundle heading as `# Lesson bundle: <id>` so the bundle's fixed text adds no en or em dash, copying user record text unchanged (REQ-change-116, #827).
- [x] Report, per affected module, the requirement IDs and spec sections an approved semantic delta declares, with a three-valued `delta_intact` flag and a derived `materialized` flag, from one memoized delta-binding read shared with the summary; keep the refusing gates live (REQ-change-113).
- [ ] Decide whether `collect_requirement_ids` (verification coverage) should move to the bounded delta read; that is a `change check` gate change and out of #826's scope.

- [x] Restrict recursive frozen-receipt capture to signed entries without changing fresh or reopen discovery and resource limits.

- [x] Implement reopen-only recorded review mismatch recovery with exact package anchoring and tamper controls.

- [x] Preserve the complete invalid recorded review, including its claimed hash, as untrusted during bounded reopen-only recovery.

- [x] Let finalize and archive read other archives structurally and validate only the history of the change they archive and of any archive they rely on; move the per-archive history walk to `change audit --archives` (REQ-change-124, #856).
- [x] Anchor archived workflow-version history pathspecs and the read-cache invalidation walk at the repository root with `:(top)`, so a project in a repository subdirectory still sees that history (REQ-change-043, REQ-change-119, #851).
- [ ] Decide whether to batch the cold archive-introduction reads (three Git processes per introducing commit) behind the closing check without loosening the partial-clone conclusiveness rules.
- [x] Record an optional relayed decision on definition approvals (portable pair included) and scoped reviews, leave it out of the review digest, and project recorded claims for status (REQ-change-relayed-decision-001, #633).
