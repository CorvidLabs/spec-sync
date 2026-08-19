## ADDED

### REQUIREMENT REQ-change-076

The effective checkout overrides SHALL be read from Git in a single configuration query rather than one query per key, and SHALL derive the same values that separate per-key queries produced.

Acceptance Criteria
- The four `core` keys that determine the checkout overrides are obtained in one `git config` invocation instead of four.
- A key set more than once resolves to its last value, matching what a single-key query returns.
- A key present with no value normalizes exactly as the empty value does.
- A key written under a mixed-case section, or with surrounding whitespace, normalizes identically.
- No matching key is treated as unset rather than as a failure.
- A malformed configuration file still fails loudly and is never read as unset, so a broken repository cannot be mistaken for a default one.
- No value is cached: every read still queries Git, so a configuration change between reads is still observed.
