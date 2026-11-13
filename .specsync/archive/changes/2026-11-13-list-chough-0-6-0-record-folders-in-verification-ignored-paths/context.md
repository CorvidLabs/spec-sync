---
change: list-chough-0-6-0-record-folders-in-verification-ignored-paths
artifact: context
---

# Context

Chough 0.6.0 adoption (2026-11-12). After `chough migrate`, `chough doctor` and the
migration receipt report `records_in_verification`: `.specsync/sdd.json` has no
`verification_ignored_paths` entry for Chough's record folders, so a Chough record
written after `specsync change check --commit` would stale the recorded verification.
Chough never writes this policy itself; its docs say adding the entries is an ordinary
reviewed change to `.specsync/sdd.json`. Leif's adoption instructions name this
follow-up as a small SpecSync change.

`.specsync/sdd.json` is both a meaningful path and a strict path in this repository, so
the change needs its own native lifecycle (scope approval, check, review, finalize).
