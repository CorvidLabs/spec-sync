---
change: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
artifact: tasks
---

# Tasks

1. Exit with the already-computed `exit_code` at the end of `run`, rather than
   returning and letting the process exit `0`.
2. Record why the line exists, in the code, so the next reader does not delete
   it as redundant: the value was computed for the body and must also reach the
   caller.
3. Verify in both directions against a failing and a passing fixture, and verify
   `--require-coverage` gates.
4. CHANGELOG entry under Fixed.
