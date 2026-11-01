## ADDED

### REQUIREMENT REQ-cmd-change-024

`change status` and `change check` SHALL disclose a non-empty `verification_ignored_paths` list in text and status JSON, together with the limit that SpecSync cannot tell whether project tests or CI read files under those prefixes, and `change ship-status` SHALL name an invalid entry as a blocker.

Acceptance Criteria
- `change status --json` carries `summary.verification_ignored_paths` when the list is non-empty and omits it otherwise.
- `change ship-status` lists an invalid `verification_ignored_paths` entry as a blocker that names the entry and the rule, not as stale verification.
- Text status and check print `Verification ignores: <entries>` and the standing note only when the list is non-empty.
- With the list absent or empty, text and JSON output are unchanged.

## MODIFIED

### SPEC SECTION Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Every status projection provides exactly one concrete next action on the `new → approve → check → review → finalize → GitHub merge` path.
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
