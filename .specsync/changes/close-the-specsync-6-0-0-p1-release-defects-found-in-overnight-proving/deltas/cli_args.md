## ADDED

### REQUIREMENT REQ-cli-args-017

`specsync check` SHALL accept a repeatable `--spec NAME` flag whose matching is identical to the positional SPEC argument. `change check` records this form as `specsync check --spec <name>`.

Acceptance Criteria
- `specsync check --spec auth --spec billing` parses as a Check command whose `spec` vector is `["auth", "billing"]`.
- Combining `--spec` with positional SPEC validates the union.
- Omitting both still validates every spec.
