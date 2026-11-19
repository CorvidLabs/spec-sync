---
change: the-relayed-decider-mismatch-error-escapes-both-names
artifact: context
---

# Context

Issue #972, a Codex P2 on #900 deferred by Leif's call. `ensure_relay_names_the_decider`
(`src/change/relayed_decision.rs`) formats `relay.decided_by` and the record's actor or
reviewer straight into its error. `check_relayed_field` allows any non-blank text up to 4096
bytes, so a relayed name can hold an escape sequence or a newline, and `change status` and
`change show` print this error to stderr on a hand-edited ledger, where it can move the cursor or
recolour the terminal. REQ-cmd-change-relayed-decision-001 says text output escapes control
characters in recorded names and relayed fields.

The fix escapes both names with `printable_record_text` (`src/change/doctor.rs`, same module),
which shows control and bidirectional formatting characters as escapes, as the doctor's output
does. The write path and the read path share this one function, so both are covered. No
requirement text changes.
