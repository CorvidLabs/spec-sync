## ADDED

### REQUIREMENT REQ-cmd-change-012

Commands that stage the whole worktree SHALL apply the sequence-ledger floor before staging, and SHALL NOT block the author when they do.

Acceptance Criteria
- Materialize, verification-evidence and archive commits all floor the ledger before `git add -A`.
- A change whose ledger went stale while its branch sat still completes, because the author caused nothing and blocking them would punish a race they cannot observe.
- The disclosure appears on standard error rather than standard output, so `--format json` output remains a single parseable document.
