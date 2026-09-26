## MODIFIED

### REQUIREMENT REQ-cmd-change-012

Lifecycle commits SHALL apply the sequence-ledger floor before staging, and SHALL NOT block the author when they do.

Acceptance Criteria
- Materialize, verification-evidence and archive commits all floor the ledger before staging.
- A change whose ledger went stale while its branch sat still completes, because the author caused nothing and blocking them would punish a race they cannot observe.
- The disclosure appears on standard error rather than standard output, so `--format json` output remains a single parseable document.

## ADDED

### REQUIREMENT REQ-cmd-change-017

`change check --commit` and `change ship --push` SHALL commit only the project's tracked edits and the untracked paths the change domain reports the change owns, and SHALL NOT stage any other untracked file.

Acceptance Criteria
- Staging uses explicit literal pathspecs; no lifecycle commit runs `git add -A`.
- An untracked file outside the change's owned paths is absent from the materialize, verification-evidence and archive commits and from what `--push` publishes, and it stays untracked and unmodified in the working tree.
- The change's own untracked workspace, its archive package, the canonical spec files its deltas write, the lifecycle ledgers, and every tracked edit are still committed, so the committed tree is the tree that was verified.
- Each untracked file left out is listed once per run on standard error, bounded for a long list, with guidance to `git add` what belongs to the delivery and to move, delete or ignore the rest; after `check --commit` the guidance also says that removing one stales the recorded verification and names the command that re-records it. Standard output under `--format json` stays a single document.
- Lifecycle runtime files, the project lock and the transaction journal, are neither staged nor listed.
- Unmerged entries are not staged, so an unresolved conflict still stops the commit instead of being recorded as resolved.
- `check --commit` makes no commit unless its first verification passes. When the re-verification against the committed tree fails, the materialize commit stays on the branch, and the error names that commit and the command that resumes.
