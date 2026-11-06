---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: tasks
---

# Tasks

- [ ] Red tests for both caches against a pre-#878 identity (committed with the draft)
- [ ] build.rs fingerprint; Cargo.toml build-dependency and package include
- [ ] Snapshot digest frames the fingerprint; another-build and same-build tests
- [ ] Read-cache identity carries the fingerprint
- [ ] Mutants: fingerprint dropped from each binding kills its test
- [ ] End-to-end repro re-run: the current build reports the duplicate
- [ ] Specs, CHANGELOG

After `change check --commit` (plain text, not tasks): `fledge lanes run verify` and `fledge lanes run pre-push` on the pinned toolchain.

Human gates (plain text, not tasks): scope approval and implementation review come from Leif through the orc (t18).
