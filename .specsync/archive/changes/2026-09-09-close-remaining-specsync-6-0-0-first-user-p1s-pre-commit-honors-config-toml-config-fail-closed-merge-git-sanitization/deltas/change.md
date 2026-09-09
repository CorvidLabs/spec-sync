## ADDED

### REQUIREMENT REQ-change-100

A workflow-v1 change in `Verifying` SHALL advertise `specsync change verify` then `specsync change accept` then `specsync change archive` as `next_action`, never the workflow-v2 `check` / `review` / `finalize` verbs.

Acceptance Criteria
- `summarize_change` for `workflow_version < 2` and `ChangeState::Verifying` (artifacts complete, approval valid) names `verify`, `accept`, and `archive` in that order.
- The `Accepted` arm is unchanged: v2 still names `finalize`; v1 still names `archive` and does not skip `accept`.
- A workflow-v2 Verifying change still names `check` / `review` / `finalize`.

### REQUIREMENT REQ-change-101

The uncovered-paths remediation SHALL spell `specsync change new` with `--kind bug-fix`, which is the clap value `ChangeKind` accepts.

Acceptance Criteria
- The message contains `--kind bug-fix` and does not contain `--kind fix`.
- `--no-spec-change` and `--path` guidance is unchanged.
