---
change: the-release-tests-exercise-the-race-guards-in-the-shipped-profile
artifact: design
---

# Design

The three guards compare a retained directory's identity (device and inode on Unix) with
the identity reopened through the public path or route. A test that renames the directory
away and creates a new one at the same path forces a different identity deterministically,
which is the state a lost race leaves. That reproduces the race outcome without timing,
so no debug-only rendezvous (#581) is involved and the tests compile under `--release`.

The tests are Unix-only, like the existing race tests; Windows is not a supported target
as of 6.0. They live beside each guard: a `release_race_guard_tests` module in
`src/validator/consumed_by_re.rs` and `src/commands/generate.rs`, and a test in
`src/mcp/confinement/tests.rs`. They share the `release_race_guard` name prefix, so one
filter selects them for the release-candidate lane.
