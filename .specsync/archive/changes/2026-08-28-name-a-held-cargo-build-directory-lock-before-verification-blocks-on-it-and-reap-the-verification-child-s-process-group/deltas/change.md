## MODIFIED

### REQUIREMENT REQ-change-091

Verification SHALL report a Cargo build-directory lock that is provably held before running the
command that will wait on it, and SHALL run every verification command as a child that leads its
own process group on Unix.

Acceptance Criteria
- The wait notice is emitted only when a non-blocking exclusive acquisition of the resolved
  `.cargo-lock` reports contention, so nothing about it is derived from elapsed time and it cannot
  fire on a slow but healthy compile.
- The notice names the lock path and states that the command is blocked rather than compiling, and
  names a holding PID only on a platform that reports lock ownership. On Unix, where it cannot name
  the holder, it names a command that narrows the holder and says what that command actually
  answers. The notice is ADDITIVE to Cargo's own `Blocking waiting for file lock on artifact
  directory` line, which reports the wait without naming the file, the holder, or a remedy.
- A Cargo command whose build directory cannot be derived exactly from its arguments and the
  process environment produces no notice at all, because naming a lock the command will never wait
  on restores the ambiguity the notice exists to remove. Underivable includes a Cargo configuration
  file in scope whose `[build]` table sets `target-dir`, `target`, or `build-dir`, or whose `[env]`
  table sets a variable this derivation reads, or that cannot be parsed.
- A command that takes no Cargo build-directory lock is never probed and never reported against.
- A verification child leads its own process group, and that group is ended when the parent unwinds
  or receives one of the interrupt and termination signals verification forwards, so an interrupted
  check cannot outlive itself holding the lock. A `SIGKILL`ed parent still orphans its child, which
  is why the notice is not optional.

### SPEC SECTION Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell, rejects direct or indirect entry into every lifecycle command surface, runs each command as a child leading its own process group on Unix so an interrupted parent can end the whole group, and names a provably held Cargo build-directory lock before the command that will wait on it starts.
6. Verification and scoped-review evidence bind the implementation commit and governed inputs; a scoped review records an explicit pass/block verdict, must be independent from the scope approver, and stays fresh only when every descendant/parent edge changes supported lifecycle persistence.
7. Invalid policy, unavailable coverage comparison, failed evidence, stale ordering gates, and protected sequence-ledger edits without lifecycle coverage fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, refuses `## ADDED` for requirement IDs already present in the living tree (agents must use `## MODIFIED`), corrupt state fails closed, and transactional same-PR finalization remains retryable before or after the archive-directory move.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
11. Concurrent effective-contract validations use isolated temporary workspaces.
12. Stale accepted delivery evidence can return only to verifying through an explicit human actor and reason, while prior verification and closing evidence remain inspectable.
13. Historical collision acknowledgements are exact immutable accepted-or-archived evidence and numeric sequence width has no four-digit upper bound.
14. A fully valid later sequence claim supersedes only the sequence-ledger bytes in historical acceptance inputs; the current owner and every other covered input remain exact evidence.
15. Supported accepted interview metadata changes only through a portable append-only correction ledger whose effective definition requires fresh gates and never replays canonical deltas.
16. Audited exact acceptance-owner corrections can repair omitted canonical ownership on an already-scoped input without changing semantic scope or replaying canonical deltas.
17. A transactional batch of audited exact acceptance-owner corrections validates every entry independently and persists all or none as sequenced ledger entries.
18. Bounded Git candidate inspection deduplicates repeated stage-zero paths only when their normalized mode and object identity match exactly; conflicting observations fail closed.
19. Only projects outside a Git repository may persist verification with no commit identity; an unborn Git repository with no `HEAD` still fails closed.
20. Workflow-v2 adoption atomically freezes a comparison-base cutoff that precedes its unique introduction, opens its lifecycle lock without following symlinks, journals only lossless UTF-8 publication paths whose filename components cannot be confused with platform separators, confines them beneath the project without symlink traversal, leaves an existing version-1 policy byte-identical, refuses to strand v1 records absent from that cutoff, routes every subsequent change through workflow v2, and fails closed if any reachable parent introduced a subsequently absent baseline.
21. Existing-change definition mutations validate correction-ledger integrity while holding the same project lock that guards persistence and return the validated effective-definition snapshot used by command output.
