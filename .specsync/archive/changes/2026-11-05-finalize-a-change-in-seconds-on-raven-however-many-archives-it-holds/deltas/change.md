## ADDED

### REQUIREMENT REQ-change-121

The system SHALL let `change finalize` and `change archive` reuse, for the duration of one command, Git answers that cannot have changed while it runs, so that one finalize does not re-derive every other archived change's history: the persistent workflow-version history verdicts and archive introductions of REQ-change-119 under the same digest and history rule, Git queries that name only full object ids, the repository prefix, and a passing workflow-v2 baseline history verdict keyed by the resolved `HEAD` id, the baseline path, its cutoff and its exact bytes.

#### Acceptance Criteria

- Every finalize and archive verdict, refusal and first error is identical to a live run.
- Only queries naming full object ids are memoized; queries naming `HEAD`, a branch or the working tree run live, `HEAD` is resolved live on every baseline check, failed Git runs are not stored, and refusals are always re-derived.
- The memo lives only inside the command's own scope and takes no working-tree snapshot, so the command sees its own writes.
- The Git processes one finalize spawns grow by at most 12 per other archived change.
