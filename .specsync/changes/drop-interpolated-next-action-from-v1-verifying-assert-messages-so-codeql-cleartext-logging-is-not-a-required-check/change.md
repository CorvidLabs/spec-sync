---
id: drop-interpolated-next-action-from-v1-verifying-assert-messages-so-codeql-cleartext-logging-is-not-a-required-check
state: implementing
type: bug_fix
base_commit: 0bdfffc065f1cd624fe272073642eb1898f8b595
---

# Drop interpolated next_action from v1 verifying assert messages so CodeQL cleartext-logging is not a required-check failure

## Intent

drop interpolated next_action from v1 verifying assert messages so CodeQL cleartext-logging is not a required-check failure

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- workflow_v1_verifying_next_action_names_verify_accept_archive keeps its substring assertions but its panic messages do not interpolate next_action; cargo test that test passes; CodeQL rust/cleartext-logging alerts 72 and 73 close

## No-spec Rationale

Not applicable
