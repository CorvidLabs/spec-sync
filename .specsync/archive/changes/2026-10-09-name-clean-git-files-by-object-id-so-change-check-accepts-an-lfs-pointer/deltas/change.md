## ADDED

### REQUIREMENT REQ-change-103

The workspace digest SHALL name each clean tracked file by its Git object id. It SHALL read file bytes only for dirty or untracked paths, and those reads SHALL stay inside the existing payload bound. `filter=lfs` SHALL be accepted as that object id. Every other content filter SHALL still fail closed. Acceptance evidence for a change SHALL still hash the bytes of the paths that change covers.

#### Acceptance Criteria
- A clean `filter=lfs` file does not fail the workspace digest, and the recorded identity is the Git object id rather than the working-tree bytes.
- A dirty copy of that file changes the workspace digest.
- `filter=demo`, `working-tree-encoding`, and `ident` still fail closed.
- Scoped acceptance of a covered path still hashes that path's bytes.

## MODIFIED

### SPEC SECTION Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. `change check` compares THIS change's specs to source in-process and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command. Scope is the modules in `affected_specs` union the specs whose `files:` fall inside `affected_paths`, so a `--no-spec-change` delivery still verifies against the contracts its source can break; drift outside that scope belongs to `specsync check`. A declared module that resolves to no spec file on disk FAILS verification and is named — never silently dropped, even when other specs are in path scope and would make the pass look real. An empty scope is a PASS only for a change that declared no module and maps no spec. Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`, where each name is what `filter_specs` matches (the file stem with `.spec` removed) and `--strict` appears only when it was requested.
6. Verification and scoped-review evidence bind the implementation commit and governed inputs; a scoped review records an explicit pass/block verdict, may be recorded by the same actor as the definition approver, and stays fresh only when every descendant/parent edge changes supported lifecycle persistence. GitHub remains the merge authority for required reviewers.
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
23. The workspace digest names each clean tracked file by its Git object id (`specsync.project-input-digest.v3`). It reads file bytes only for dirty or untracked paths, and those reads stay inside the existing payload bound. `filter=lfs` is that object id, which is the pointer, so a clean LFS file is not read from the worktree. Every other content filter still fails closed. A change's own acceptance evidence still hashes the bytes of the paths it covers.
