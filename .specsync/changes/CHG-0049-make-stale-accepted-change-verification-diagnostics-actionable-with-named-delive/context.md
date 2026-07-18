---
change: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
artifact: context
---

# Context

When a governed delivery input of an accepted change changes after closing approval, strict
checking fails closed with `accepted change verification is stale for current delivery inputs`.
The attached reason is written in internal evidence vocabulary — for example `accepted input
obligation \`src/lib.rs\` owner \`change\` has no closing-valid terminal semantic successor` — and
never says how to recover. Operators hitting this in the wild (most commonly because creating a
new change bumps the protected `.specsync/change-sequence.json` ledger that every accepted change
signs) had to read `validate_accepted_inputs_recursive` in `src/change.rs` to learn that the
remediations are either "verify and accept the successor change that covers the input" or
"`specsync change reopen <id>`".

The freshness model itself is correct and deliberately unchanged: this is a diagnostics-only
change. The stale reasons produced by `validate_accepted_inputs_recursive` are rewritten so each
one names the offending delivery input path (and owner module where applicable), distinguishes an
uncovered input from one covered by a successor whose own evidence is stale (naming those
successor change IDs in sorted order), and states the concrete remediation command. Output stays
deterministic: sorted successor IDs, no timestamps, no environment-dependent content.

Key files: `src/change.rs` (`validate_accepted_inputs_recursive` reason sites plus the
`check_project` stale-error prefix) and `tests/integration/change.rs` (CLI-level regression).
