---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: tasks
---

# Tasks

- [x] Reproduce the disagreement: the same spec scoring 80 / exit 0 and hard-failing `check`.
- [x] Establish where 80 comes from, rather than treating it as an arbitrary threshold —
      freshness 15/15 for a path that exists, API 0 for a read that failed.
- [x] Enumerate every site that decides whether a `files:` entry is a directory, before
      changing any of them.
- [x] Classify once in the export scan rather than special-casing the reported command.
- [x] Handle the new variant at every consumer so the compiler enforces coverage.
- [x] Keep `score` a metric: zero and grade F, not a hard failure, so `--explain` and JSON
      still render for the affected spec.
- [x] Leave `check` untouched — it was already correct.
- [x] Add the vacuity control: a real source file must still score at or above the strict bar,
      passing on BOTH binaries.
- [x] Revert the hand-edited `specs/` and express the contract changes as semantic deltas.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Confirm gate 059 flips and the whole board moves by exactly one.
