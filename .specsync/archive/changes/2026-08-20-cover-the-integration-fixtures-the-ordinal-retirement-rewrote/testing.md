---
change: cover-the-integration-fixtures-the-ordinal-retirement-rewrote
artifact: testing
---

# Testing

The fixtures themselves are the test. Both files assert the lifecycle end to end and are run by
`cargo test --test integration`: 405 integration tests, 0 failures, on the tree this covers.

No new assertions — the rewritten ones are already discriminating, since they previously
hard-coded `CHG-NNNN` identities that the retirement change no longer mints.
