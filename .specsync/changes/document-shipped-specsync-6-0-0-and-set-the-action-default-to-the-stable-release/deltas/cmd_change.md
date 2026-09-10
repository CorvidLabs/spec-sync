## MODIFIED

### SPEC SECTION Change Log

| 2026-07-30 | Scoped `change check`; add `change audit` two-verb UX |

| Date | Change |
|------|--------|
| 2026-09-01 | Scoped review next-action uses `--reviewer <human>`; same-actor review is allowed. |
| 2026-08-30 | `change check` is spec↔code sync; it does not spawn `verification_commands`. |
| 2026-08-30 | `ship` / archived `next_action` no longer gate merge on writing lessons. |
| 2026-08-01 | Draft next-action prefers complete artifacts over approve. |
| 2026-07-10 | Initial 5.0 change command |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-13 | Add text and deterministic JSON dispatch for audited stale-accepted reopen |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-30 | CHG-0068: Render explicit pass/block scoped-review results while keeping independence policy in the change domain |
| 2026-07-30 | CHG-0068: Render stable reviewer claims while preserving append-only attempts and externally authenticated check provenance |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-08-01 | CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa: Approve rejects living ADDED REQs and draft next_action waits on complete artifacts |
| 2026-08-07 | CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires: Add change check --commit to perform the sequence it requires |
| 2026-08-07 | CHG-0091-add-change-ship-status-for-local-ship-readiness-and-merge-before-finalize-warning: Add change ship-status for local ship readiness and merge-before-finalize warning |
| 2026-08-07 | CHG-0092-complete-buttery-ship-status-tip-class-and-ship-preflight-for-agents: Complete buttery ship status tip class and ship preflight for agents |
| 2026-08-07 | CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path: Encode multi-active ship ordering warnings and Agents.md happy path |
| 2026-08-07 | CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path: Encode ship multi-active ordering rules and AGENTS happy path |
| 2026-08-08 | CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha: Ship-status live GitHub check-run trust for product parent SHA |
| 2026-08-08 | CHG-0100-ship-push-wait-archive-tip-orchestration-for-buttery-multi-tip-ship: Ship --push --wait archive tip orchestration for buttery multi-tip ship |
| 2026-08-08 | CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid: Fail closed in text lifecycle views when a correction ledger is invalid |
| 2026-08-10 | CHG-0103: Validate correction-ledger integrity before existing-change mutations and increment the command contract version |
| 2026-08-10 | CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif: Address PR 531 review by validating correction-ledger health before mutating lifecycle commands and incrementing the cmd_change contract version |
| 2026-08-10 | CHG-0103: Delegate correction-ledger validation to locked change-domain mutations and render successful mutations from their validated transaction snapshots |
| 2026-08-10 | CHG-0103: Select the normal/strict mutation summary captured under the domain transaction lock |
| 2026-08-17 | CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent: An unreadable change workspace must be reported, not counted as absent |
| 2026-08-17 | CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards: A stale sequence ledger must not be committed backwards |
| 2026-08-18 | CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present: The sequence ledger floor must be wired, not merely present |
| 2026-08-19 | CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar: Ship-status must name the action the lifecycle state requires, and resolve an archived change's evidence |
| 2026-08-23 | ship-readiness-is-a-content-question-not-a-history-one: Ship readiness is a content question, not a history one |
| 2026-08-24 | close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails: Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize |
| 2026-08-24 | ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so: Ship must name the lesson fold-back too: the archive bundle is written and only finalize says so |
| 2026-08-29 | ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell: Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell |
| 2026-09-02 | allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second: Allow the scope approver to record scoped review so solo projects can ship when GitHub does not require a second reviewer. |
| 2026-09-02 | finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a: Finish the same-actor scoped-review user-facing copy so CLI, ADOPTING, and generated agent skills no longer demand a second identity |
| 2026-08-30 | make-check-the-product-and-stop-change-check-from-spawning-project-tests: Make check the product and stop change check from spawning project tests |
| 2026-09-02 | tell-agents-when-it-is-safe-to-clear-context: Tell agents when it is safe to clear context |
| 2026-09-07 | make-ship-status-product-stage-completion-use-current-verification-content-consistently: Make ship-status product-stage completion use current verification content consistently |
| 2026-09-10 | document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release: Document shipped SpecSync 6.0.0 in ADOPTING (stable install, Trust 1.2.0, always pass Action version) |
