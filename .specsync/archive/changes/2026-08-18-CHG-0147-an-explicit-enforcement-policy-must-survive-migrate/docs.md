---
change: CHG-0147-an-explicit-enforcement-policy-must-survive-migrate
artifact: docs
---

# Docs

## User-visible change

A configuration written by `migrate` now always contains an `enforcement` line.
Previously the line was omitted when the value was `warn`, and the project
silently adopted whichever default the binary carried — `strict` since 6.0.

## Upgrade note

A project that set `enforcement = "warn"` and has ALREADY migrated on an
affected build is currently running as `strict` without saying so. Re-add the
line, or re-run `migrate` on this build, which now preserves it.

The documented default in the configuration reference was also wrong: it said
`warn`, and the default is `strict`.

## Unchanged

Projects that never expressed a preference, and projects on `enforce-new` or
`strict`, are unaffected.
