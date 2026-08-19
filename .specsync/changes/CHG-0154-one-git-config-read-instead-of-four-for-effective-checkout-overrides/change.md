---
id: CHG-0154-one-git-config-read-instead-of-four-for-effective-checkout-overrides
state: implementing
type: feature
base_commit: 03210d94dbf5993692edb302ef3f399b77bcf787
---

# One git config read instead of four for effective checkout overrides

## Intent

one git config read instead of four for effective checkout overrides

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reading the effective checkout overrides issues one git subprocess instead of four, by asking for the four core keys in a single --get-regexp query rather than four --get queries; the values derived are identical to the four-query result for every case git distinguishes, including a multi-valued key resolving to its last value, a valueless key normalizing as the empty value does, a mixed-case section name, surrounding whitespace, no key set at all, and a malformed config file which must still fail loudly rather than read as unset; nothing is cached, so each call still spawns and a configuration change between calls is still observed; and core.fsmonitor is deliberately left on its own path because it is read through a command that scrubs system, global and injected configuration, so folding it in would silently change how it resolves.

## No-spec Rationale

Not applicable
