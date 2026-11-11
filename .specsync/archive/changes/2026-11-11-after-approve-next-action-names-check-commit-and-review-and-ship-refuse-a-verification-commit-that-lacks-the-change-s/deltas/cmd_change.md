---
module: cmd_change
---

## MODIFIED

### SPEC SECTION Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Every status projection provides exactly one concrete next action on the `new → approve → check --commit → review → finalize → GitHub merge` path.
4. Supersede records an explicit digest-bound predecessor/path/module obligation before definition approval.
5. Reopen renders the exact persisted versioned supersession event in deterministic JSON.
6. Correct-owner renders one persisted exact canonical-owner correction and directs the user to definition reapproval.
7. Batch correct-owner resolves repeated paths, a manifest, or `--all-missing` into domain entries, renders the persisted record, and directs the user to definition reapproval without partial mutation on failure.
8. Strict is an additive validator selection on the same workflow and evidence, never a second lifecycle mode.
9. Finalize reports readiness for GitHub merge and never invokes an external merge API.
10. Review renders the same explicit pass/block verdict and stable reviewer claim in text and JSON
    while domain policy preserves append-only attempts and hosted policy authenticates provenance.
11. Existing-change mutations delegate to domain transactions that validate correction-ledger
    integrity after acquiring the persistence lock and render from the returned validated snapshot,
    so neither rendering failure nor a lock-wait race can conceal a mutation that already took effect.
12. Status and check render a non-empty `verification_ignored_paths` list in text and status JSON with the standing note that SpecSync cannot tell whether project tests or CI read those files, render nothing extra when the list is absent or empty, and ship-status names an invalid entry as a blocker.
13. Text `change check` (plain and `--commit`) and a passing text `change audit` print one stderr note counting the verification commands configured in `.specsync/sdd.json` that they did not run — `verification_commands` first, then `component_verification_commands` by module, then `strict_verification_commands`, each command once — and JSON check and a passing JSON audit carry the same deduplicated list as top-level `configured_commands_not_run` (`[]` when none is configured). The disclosure never spawns a configured command and never changes a verdict or exit status; `change check --commit --json` prints its own outcome document, which carries the same list (REQ-change-004, REQ-cmd-change-unrun-commands-001, #839).
