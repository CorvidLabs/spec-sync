---
change: CHG-0159-identity-must-come-from-state-json-never-from-the-shape-of-a-name
artifact: context
---

# Context

This is step 1 of the change-identity work. The owner has accepted a schema break to retire the
`CHG-NNNN` ordinal and keep only the slug, and a migration is being designed separately.

Step 1 is deliberately first and deliberately independent of that design. Two gates currently
decide identity from the shape of a name, and both fail open. One is broken *today* — the
undated `CHG-0001-foo` form already slips through — and the other becomes broken the moment the
ordinal moves. Fixing them before the identity work means the gates are shape-independent
before there is a new shape, rather than being repaired after they have already stopped firing.

The mandatory-review one is the reason this could not wait. A gate that stops running makes CI
go green *faster*, so nothing about the symptom suggests a problem.

One thing this change deliberately does not do: it does not make `is_positive_legacy_tombstone`
fully shape-independent. It cannot, yet — a dated package reduced to `deltas/` alone is refused
purely because of its name, and no content signal distinguishes it from a genuine legacy
tombstone. That gap is recorded in the code comment beside signal 3 so the migration inherits a
stated problem rather than a silent one.
