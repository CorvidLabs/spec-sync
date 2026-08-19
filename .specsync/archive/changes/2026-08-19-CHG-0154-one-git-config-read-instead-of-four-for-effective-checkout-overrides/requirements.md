---
change: CHG-0154-one-git-config-read-instead-of-four-for-effective-checkout-overrides
artifact: requirements
---

# Requirements

## REQ-change-076 (new)

The effective checkout overrides SHALL be read from Git in a single configuration query rather
than one query per key, and SHALL derive the same values that separate per-key queries produced.

See `deltas/change.md` for the canonical delta.

## Deliberately unchanged

Every value derived and every error surfaced. This is a read pattern, not a behaviour change —
which is why the tests for it must pass identically before and after.

`core.fsmonitor` keeps its own path, because it resolves through a command that scrubs system,
global and injected configuration.
