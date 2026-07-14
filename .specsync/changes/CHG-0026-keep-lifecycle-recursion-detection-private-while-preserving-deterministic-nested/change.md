---
id: CHG-0026-keep-lifecycle-recursion-detection-private-while-preserving-deterministic-nested
state: accepted
type: refactor
base_commit: a6706a5611b56d5998de59585386b6cec40b095e
---

# Keep lifecycle recursion detection private while preserving deterministic nested-command failures

## Intent

Keep lifecycle recursion detection private while preserving deterministic nested-command failures

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Nested lifecycle commands still fail once with the established deterministic recursion error; the recursion helper and context marker are not exported from the change module; canonical public API documentation contains no internal recursion helper.

## No-spec Rationale

Not applicable
