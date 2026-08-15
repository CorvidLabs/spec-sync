---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: research
---

# Research

The sweep was for the WRAPPER, not the symptom. Searching for "score grades the
wrong surface" finds `scoring.rs`. Searching for callers of
`scan_exported_symbols` finds five commands, four of which nobody had reported.

That is the fifth time in this campaign the reported site was one of several,
and the second where the sweep turned a single-command bug into a
cross-command contradiction — after #572, where a staleness fix in `stale.rs`
turned out to be missing from three other readers.

Companion prose in several specs still names the old entry points
(`specs/cmd_new/requirements.md:18,44`, `specs/cmd_scaffold/cmd_scaffold.spec.md:64`,
`specs/cli/cli.spec.md:325`, `specs/validator/validator.spec.md:220`). `check`
does not validate those tables, so they are stale prose rather than drift, and
are left for a documentation pass rather than widened into this change.
