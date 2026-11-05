---
change: make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero
artifact: tasks
---

# Tasks

- [x] Capture Leif's #841 review nits verbatim in hi as CHANGE-10.a, CHANGE-10.b and CHANGE-10.c.
- [x] Commit failing tests before each fix: sorted undeclared listers and the `_ =>` arm (cf69960), the depth-0 fallback and `src/foo` vs `src/foobar` (72675b7).
- [x] Collect every spec that lists the path and sort by spec path; split the `_ =>` arm into its two cases.
- [x] Guard on `depth > 0` and fall back to the generic message.
- [x] Record REQ-change-120 coverage in specs/change/testing.md.
