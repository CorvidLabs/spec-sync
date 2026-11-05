---
id: make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero
state: archived
type: bug_fix
base_commit: 5e137ece36437d8eaa5b54f481457d8f7444ee49
---

# Make the unowned-path ownership hint deterministic and fall back at depth zero

## Intent

Make the unowned-path ownership hint deterministic and fall back at depth zero

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- `ownership_hint` picks the first declared spec even at depth 0, so `lib/new.ts` can be pointed at `specs/billing/billing.spec.md`. Guard on `depth > 0` and fall back to the generic message.
- The first-pass walk returns the first undeclared spec that lists the path, in `read_dir` order. That order isn't deterministic across platforms. Collect every match and sort.
- Missing tests: the depth-0 fallback, `src/foo` vs `src/foobar`, a glob entry, and the `_ =>` arm.

## No-spec Rationale

Not applicable
