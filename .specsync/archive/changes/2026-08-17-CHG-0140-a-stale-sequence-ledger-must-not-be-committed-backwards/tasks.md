---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: tasks
---

# Tasks

- [x] Reproduce through the real CLI rather than from reading the code.
- [x] Confirm the reproduction matches the issue's live evidence, not merely its shape.
- [x] Enumerate every caller of `git_commit_all` before changing anything (three: materialize,
      verification evidence, archive).
- [x] Put the floor at the shared staging point so a later commit path cannot bypass it.
- [x] Raise rather than refuse, and return the changed pair so the caller can disclose it.
- [x] Write the ahead-of-mark control, which is what stops the fix becoming "always restore".
- [x] Merge acknowledged collisions across the raise rather than taking one side.
- [x] Disclose on stderr so it survives `--quiet` and stays off a JSON stdout payload.
- [x] Unit tests: raise-stale, leave-ahead-alone, equal-not-reported.
- [x] CLI discrimination on two genuinely different binaries.
- [x] Find the paired pin drill by READING drills, not by grepping the issue number.
- [x] Invert drill 037 and repair the three fixture defects it had been masking.
- [x] Verify the inverted drill fails on an origin/main binary with the high-water diagnostic.
- [x] Whole-board check: gate 051 stayed RED after the commit-path fix, because it asserts
      TWO behaviours. Read the gate rather than assuming the issue's headline was the whole
      requirement.
- [x] Fix the second half: `validate_change_sequences` floors against the default branch's
      published mark, reusing `remote_sequence_high_water` rather than re-implementing it.
- [x] Confirm the refusal does not break the five call sites it is reachable from.
- [x] Re-run the whole board with both halves: pass=46 fail=9, up from 45/10. Gate 051 flipped,
      drill 037 stayed PASS, nothing else moved.
- [x] Semantic deltas for both modules; no hand-edited `specs/`.
- [x] CHANGELOG entry disclosing the new stderr note and the manual-git limit.
