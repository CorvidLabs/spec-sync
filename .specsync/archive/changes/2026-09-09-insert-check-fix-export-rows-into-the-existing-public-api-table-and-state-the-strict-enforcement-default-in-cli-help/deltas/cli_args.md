## ADDED

### REQUIREMENT REQ-cli-args-016

The `--enforcement` help summary SHALL name `strict` as the default mode, consistent with the accepted-value descriptions and the 6.0 default.

Acceptance Criteria
- `specsync check --help` describes `--enforcement` as strict by default, with `enforce-new` and `warn` as the opt-in alternatives.
- Argument grammar, accepted values, and runtime enforcement behaviour are unchanged.
