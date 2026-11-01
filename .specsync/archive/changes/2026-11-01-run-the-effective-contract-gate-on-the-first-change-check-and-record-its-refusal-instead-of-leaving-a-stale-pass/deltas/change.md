## ADDED

### REQUIREMENT REQ-change-112

The system SHALL apply the effective-contract gate to the change being verified on every `change check`, including the first one after materialization, and SHALL record that gate's refusal of a module the change declares as a failed verification attempt, so recorded verification never reports a pass the lifecycle's own gates refuse for the same tree.

Acceptance Criteria
- The first `change check` of an approved change whose declared module carries an unsuppressed validator warning fails and names the warning, as later checks, finalization, and audit already do, whatever `enforcement` is configured.
- The refusal is appended to `verification-attempts.json` and projected into `verification.json` with `passed: false`; `commands[]` keeps the scoped `specsync check --spec <name> …` entry and adds `specsync change check <id> (effective contract: <module>, …)` with `success: false`.
- A failed attempt supersedes an earlier recorded pass, so status names `specsync change check <id>` rather than review, and `change check --commit` commits nothing.
- A change whose declared modules carry no unsuppressed warning records exactly the single scoped command it recorded before.
- Gate failures not attributable to a module the change declares are refused before any attempt is recorded, as before.
- `specsync check`, configured `enforcement`, explicit `--strict`, the gate's promotion rule and suppressions, path-only scope, `--no-spec-change` changes, acceptance, audit, and accepted or archived evidence are unchanged.

## MODIFIED

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
- When the effective-contract gate refuses a module the change declares, the failed attempt keeps
  that scoped entry and adds `specsync change check <id> (effective contract: <module>, …)` with
  `success: false`, the command that reproduces the gate's refusal, rather than claiming a
  `--strict` run that did not happen.
- `change check` does not execute `.specsync/sdd.json` `verification_commands`.
- Direct re-entry into SpecSync through `SPECSYNC_VERIFICATION_CONTEXT` still fails once.
- Failed spec↔code attempts remain inspectable and a corrected retry can record passed latest evidence.
- Other failed or stale changes continue failing closed.

### SPEC SECTION Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. `change check` compares THIS change's specs to source in-process and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command. Scope is the modules in `affected_specs` union the specs whose `files:` fall inside `affected_paths`, so a `--no-spec-change` delivery still verifies against the contracts its source can break; drift outside that scope belongs to `specsync check`. A declared module that resolves to no spec file on disk FAILS verification and is named — never silently dropped, even when other specs are in path scope and would make the pass look real. An empty scope is a PASS only for a change that declared no module and maps no spec. Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`, where each name is what `filter_specs` matches (the file stem with `.spec` removed) and `--strict` appears only when it was requested. The effective-contract gate judges the change being verified on every check, the first one after materialization included, and a refusal of a module the change declares is recorded as a failed attempt that also names `specsync change check <id> (effective contract: <module>, …)`.
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
23. The workspace digest names each clean tracked file by its Git object id (`specsync.project-input-digest.v4`), in an `object-id` frame that file bytes cannot match. It reads file bytes only for dirty or untracked paths, and those reads stay inside the existing payload bound. `filter=lfs` is that object id only when the configured clean and smudge commands are Git LFS and the blob is an LFS pointer. Every other content filter still fails closed. A change's own acceptance evidence still hashes the bytes of the paths it covers.
