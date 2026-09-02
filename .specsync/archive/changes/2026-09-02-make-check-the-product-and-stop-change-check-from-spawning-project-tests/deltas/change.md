## MODIFIED

### SPEC SECTION Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. `change check` compares THIS change's specs to source in-process and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command. Scope is the modules in `affected_specs` union the specs whose `files:` fall inside `affected_paths`, so a `--no-spec-change` delivery still verifies against the contracts its source can break; drift outside that scope belongs to `specsync check`. A declared module that resolves to no spec file on disk FAILS verification and is named — never silently dropped, even when other specs are in path scope and would make the pass look real. An empty scope is a PASS only for a change that declared no module and maps no spec. Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`, where each name is what `filter_specs` matches (the file stem with `.spec` removed) and `--strict` appears only when it was requested.
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
20. Workflow-v2 adoption atomically freezes a comparison-base cutoff that precedes its unique introduction, opens its lifecycle lock without following symlinks, journals only lossless UTF-8 publication paths whose filename components cannot be confused with platform separators, confines them beneath the project without symlink traversal, leaves an existing enabled version-1 policy byte-identical (adoption rewrites only `enabled`, and only when it is off), refuses to strand v1 records absent from that cutoff, routes every subsequent change through workflow v2, and fails closed if any reachable parent introduced a subsequently absent baseline.
21. Existing-change definition mutations validate correction-ledger integrity while holding the same project lock that guards persistence and return the validated effective-definition snapshot used by command output.

### REQUIREMENT REQ-change-023

Verification SHALL compare THIS CHANGE's specs to code in-process, SHALL NOT spawn project
test or build commands, and SHALL preserve retryable attempt history without weakening
unrelated gates.

Acceptance Criteria

- Scope is the union of the modules in `affected_specs` and every spec whose `files:` mapping
  falls inside a declared `affected_paths` scope. Drift outside that scope does not fail this
  change; project-wide validation is `specsync check`.
- A declared module that resolves to no spec file on disk FAILS verification and is named in the
  error, and the attempt is recorded like any other spec↔code failure so the retry after writing
  the spec stays append-only. It is never dropped from scope, even when other specs are in path
  scope and would otherwise make the pass look real.
- An empty scope is a PASS only for a change that declared no module and whose declared paths map
  no spec — a change that claimed no contract.
- Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`,
  each name being what `filter_specs` matches (the file stem with `.spec` removed) rather than a
  frontmatter `module:` that would select nothing, with `--strict` only when requested. It
  reproduces the verdict when every named spec resolves or when none does; a mixed scope fails
  the check but can rerun green, because an unmatched filter is demoted to a warning once any
  filter matches.
- `change check` does not execute `.specsync/sdd.json` `verification_commands`.
- Direct re-entry into SpecSync through `SPECSYNC_VERIFICATION_CONTEXT` still fails once.
- Failed spec↔code attempts remain inspectable and a corrected retry can record passed latest evidence.
- Other failed or stale changes continue failing closed.

### REQUIREMENT REQ-change-049

Lifecycle verification SHALL resolve evidence completeness before comparing specs
to code, SHALL name the artifact and section an author must edit to close an
evidence gap, and SHALL NOT spawn the project's test or build commands. Spec↔code
sync is the verifier. Delta application
SHALL converge when an `## ADDED` block is already present with byte-identical content, and
SHALL reject a duplicate `CHG-NNNN` ordinal claimed by two distinct changes from the same
base commit.

Acceptance Criteria

- Incomplete acceptance or requirement evidence fails before spec↔code sync runs.
- The evidence-gap message names the change `testing.md` and its `## Requirement evidence`
  table. Drift failure names the spec finding, not a test-suite exit code.
- Configured `verification_commands` in `.specsync/sdd.json` are not executed.
- An `## ADDED` block already present with byte-identical content applies as a no-op, so
  re-deriving the canonical tree converges.
- An `## ADDED` block present with different content fails and directs the author to
  `## MODIFIED`.
- Two distinct changes claiming one ordinal from the same base commit are rejected at
  definition approval and by `change audit`; differing or unknown base commits are accepted.

### REQUIREMENT REQ-change-050

SpecSync SHALL leave a newly initialised project able to complete its own lifecycle, and
SHALL treat an active-change directory that contains no `state.json` as not an active change
in this working tree rather than as corruption.

Acceptance Criteria

- Fresh `init` writes SDD off with an empty `verification_commands` list; `specsync check` is
  the next step and does not need a project test command.
- A change directory with no `state.json` is skipped by active-change discovery, so
  `change new` succeeds on a branch that does not contain an earlier change.
- Every other read error, including an unreadable or malformed `state.json`, still fails closed.
- Verification exposes a lock-free body so a caller already holding the project lock can
  re-run it without deadlocking on the non-reentrant lock.

### REQUIREMENT REQ-change-058

The lifecycle check SHALL NOT spawn configured verification commands, and the
quiet-output variant used solely to keep lifecycle findings out of a machine-consumed
report stream SHALL NOT exist.

Acceptance Criteria

- `change check` and `change audit` do not execute `.specsync/sdd.json` `verification_commands`.
- The quiet-output check path and its selector type are absent rather than retained
  unused, so no caller can reintroduce suppressed-output command execution.
- Failed spec↔code evidence remains inspectable in `verification.json`.

### REQUIREMENT REQ-change-091

Lifecycle verification SHALL NOT spawn project test or build commands, so it SHALL NOT wait on a
Cargo build-directory lock and SHALL NOT create a verification child process.

Acceptance Criteria
- `change check` records in-process spec↔code evidence naming its scope, as
  `specsync check --spec <name> …`.
- A configured `verification_commands` sentinel is not executed.
- A held Cargo `.cargo-lock` is not named on stderr during `change check`.
- A configured reporter script is not started.
