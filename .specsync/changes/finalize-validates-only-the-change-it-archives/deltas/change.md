## ADDED

### REQUIREMENT REQ-change-124

The system SHALL let `change finalize` and `change archive` validate in full only the change they archive, reading every other archived change structurally from its own `state.json` bytes without walking its Git workflow-version history, and SHALL keep that per-archive history walk in `change audit --strict`, which names each archived change whose workflow-version history fails.

#### Acceptance Criteria

- The Git processes one finalize spawns do not grow with the number of other archived changes once the persistent read cache is warm. With a cold cache, each other archive adds at most the three reads of the immutable commit that introduced it, and no workflow-version history walk.
- Every refusal finalize and archive issue for the change being archived is unchanged: its own workflow-version history, evidence, review, deltas, ownership, closing approval and archive integrity are still validated in full.
- An archived change that finalize or archive relies on, as a dependency, a semantic successor or a sequence owner, still has its own workflow-version history validated before it is relied on.
- `change audit --strict` refuses a workflow-version downgrade committed into another archive's history and hidden again by a later commit, naming that change, with or without a warm read cache. `change audit` without `--strict` is unchanged.
