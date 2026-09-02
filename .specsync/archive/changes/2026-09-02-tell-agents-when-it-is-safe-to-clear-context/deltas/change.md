## ADDED

### REQUIREMENT REQ-change-093

The lifecycle SHALL compute a handoff readiness for every change so an agent can tell whether
clearing its context — or handing the change to a fresh session — loses anything the lifecycle
still needs, and SHALL say what to do first when it does.

Acceptance Criteria
- `handoff_summary` returns `safe`, `conditional`, or `not-yet` with a plain-language reason, the
  resume command `specsync change status <id>`, and, when readiness is not `safe`, at least one
  concrete step to take before clearing. No reason or step contains a digest.
- `classify_handoff` is a pure function of `HandoffSignals`; every branch has a unit test that
  needs no repository.
- A frozen sequence ledger, a definition changed after its approval, an invalid correction
  ledger, and stale legacy terminal evidence are `not-yet`, and the step named is the repair the
  lifecycle already requires (clear the freeze, re-approve, restore the ledger, reopen).
- A Draft is never `safe`: open questions, stub artifacts, and a complete-but-unapproved
  definition are each `conditional`, and the steps name answering, finishing the artifacts, or
  approving — approval is the first boundary a fresh session can resume from.
- Uncommitted edits under the change's `affected_paths` make an approved, implementing, or
  verifying change `conditional` and name committing or writing the intent into `change.md`;
  uncommitted files under `.specsync/` alone never do, because `change review` then
  `change finalize` runs with that evidence uncommitted by design.
- A verifying change whose recorded verification is stale is `conditional` and names
  `specsync change check <id> --commit`; one whose verification is current is `safe` whether the
  scoped review has been recorded yet or not, and the reason says which step a fresh session
  resumes at.
- An accepted workflow-v2 change and an archived change are `safe`.
- `ChangeSummary` carries the decision as `handoff`, so JSON consumers read the same verdict the
  text line prints.

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
22. Every change carries a handoff readiness — `safe`, `conditional`, or `not-yet` — computed as a pure function of its lifecycle signals: a project-wide sequence freeze, open interview questions, artifact completeness, definition-approval currency, correction-ledger validity, uncommitted edits under `affected_paths` (never `.specsync/` evidence, which the review → finalize pair leaves uncommitted by design), verification currency, scoped-review currency, and terminal-evidence staleness. A Draft is never `safe` because approval is the first boundary a fresh session can resume from; the summary names one plain-language reason without digests, the resume command `specsync change status <id>`, and the steps to take before clearing when readiness is not `safe`.
