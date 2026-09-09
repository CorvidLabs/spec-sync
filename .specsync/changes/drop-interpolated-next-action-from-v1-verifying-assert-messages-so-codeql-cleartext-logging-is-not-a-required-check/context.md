---
change: drop-interpolated-next-action-from-v1-verifying-assert-messages-so-codeql-cleartext-logging-is-not-a-required-check
artifact: context
---

# Context

PR #774 is merge-blocked by the CodeQL required check: 2 high `rust/cleartext-logging` alerts (#72, #73) on `src/change_tests.rs` lines 16596 and 16600.

Those lines are panic messages on `workflow_v1_verifying_next_action_names_verify_accept_archive` that interpolate `{next}` (`summarize_change(...).next_action`). CodeQL taint-tracks that string as sensitive. The substring assertions themselves are fine.

Drop the interpolations. Do not change product code or the assertions.
