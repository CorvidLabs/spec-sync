---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: testing
---

# Testing

Discrimination against a binary built from `origin/main` in a separate
worktree — not against a partially-reverted tree, which is how this nearly went
wrong:

    drill 058   unfixed  pass=6  fail=0 pending=2     fixed  pass=8  fail=0 pending=0
    drill 038   unfixed  pass=11 fail=1               fixed  pass=12 fail=0
    full board  before   pass=41 fail=14              after  pass=42 fail=13

**Recorded because it produced a false proof first.** Reverting only
`hash_cache.rs` and `commands/check.rs` to build an "unfixed" binary failed
to compile — other files referenced the reverted code — so the drills ran
against the still-fixed binary and both passed. That reads exactly like
successful discrimination. It was caught by checking the build's exit status
(101) rather than the drill's. A real unfixed binary from `origin/main` gave
the boards above.

Drill 038 had to move in the same change: it pinned the buggy behaviour and said
so in its own failure text. Its inverted assertion fails on the unfixed binary,
which is what makes the inversion worth having.

Suite: fmt clean, clippy clean, 2275 unit + 374 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-hash-cache-004 | Gate 058 goes 2 pending to 0. The stored result is written where the snapshot types already existed but were never wired — the fix is store-then-replay, established by asking whether findings were never stored or stored and not replayed before any code was written |
| REQ-cmd-check-013 | The warm run now names `Undocumented export 'sub'` in text and reports `specs_checked: 1` in JSON, matching the cold run. The in-sync control still reports clean, so replay does not manufacture findings |
| REQ-commands-012 | The whole board moved by exactly one drill, which is the evidence that no other command's verdict shifted with the cache state |
