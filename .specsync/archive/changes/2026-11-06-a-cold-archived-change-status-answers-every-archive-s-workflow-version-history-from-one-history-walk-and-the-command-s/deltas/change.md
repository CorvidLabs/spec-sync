## ADDED

### REQUIREMENT REQ-change-140

Inside a change read scope, the workflow-version history of every archived change SHALL be answered from one lifecycle-state history index per command, built from a fixed number of Git walks from `HEAD` over the lifecycle state paths of the changes root and the archive root. Each archive's candidate state paths, its oldest-first state-path history and its history commits' parents SHALL be derived from that index, and every archive SHALL still be fully validated. A cold archived `change status` SHALL NOT spawn more Git processes as the project holds more archives (SPEED-1.a.1). The index SHALL never change a verdict: each archive SHALL be accepted or refused exactly as the per-archive check would, with the same first refusal and message, the same non-canonical archive path refusal, and the same per-archive history and parent bounds. When the index cannot be built (over its commit bound or output limit, or a Git call fails), each archive SHALL fall back to the per-archive check.

Acceptance Criteria
- `cold_archived_status_git_processes_do_not_grow_with_archives`: four more archives add fewer than 3 Git processes to a cold archived status.
- Index-derived candidate paths, history and parents equal the per-archive Git answers on histories with merges, moved or re-archived packages, root commits, a project in a subdirectory, a non-canonical archive path, a deleted creation anchor, a changed identity and a history over the per-archive bound.
- With the index unavailable, every verdict equals today's.

### REQUIREMENT REQ-change-141

When the lifecycle-state history index answers, the state-file reads and path-exists checks behind archived workflow-version validation SHALL run on the command's own thread through its one object batch reader (REQ-git-utils-010), with a read deadline check between archives (REQ-change-137). A state blob the commit's tree lists but Git cannot read SHALL still refuse, and SHALL never be treated or cached as absent. Prefetch workers (REQ-git-utils-011) SHALL remain the path when the index does not answer.

Acceptance Criteria
- A listed state blob that cannot be read refuses the archive and leaves no cached absence.
- A cold archived status past its `--timeout` still exits 124 with the phase and progress.
