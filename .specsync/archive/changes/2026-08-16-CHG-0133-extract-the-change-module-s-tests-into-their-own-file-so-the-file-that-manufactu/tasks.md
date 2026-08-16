---
change: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
artifact: tasks
---

# Tasks

1. Extract lines 17459..29982 — the body of `mod tests` — into
   `src/change_tests.rs`.
2. Replace the block with the `#[cfg(test)] #[path]` declaration.
3. Leave the 24 test-only helpers in place.
4. Verify by COUNTING, not by reading the diff.
