---
change: CHG-0147-an-explicit-enforcement-policy-must-survive-migrate
artifact: tasks
---

# Tasks

- [x] Reproduce: same tree, same findings, check rc 0 before migrate and 1 after.
- [x] Confirm the cause is an omit-on-default against a literal, not a parse bug.
- [x] Write the sandbox pin BEFORE the fix, and confirm it fails on the unfixed binary.
- [x] Fix a false PASS in that drill (`grep -c` prints 0 and exits 1, so `|| echo 0`
      appended a second zero and the assertion stopped comparing equal).
- [x] Emit the key unconditionally rather than comparing against an assumed default.
- [x] Correct the documented default.
- [x] Confirm the drill flips and the whole board moves by exactly one.
