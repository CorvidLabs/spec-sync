---
change: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership
artifact: tasks
---

# Tasks

- [x] Refuse a passing scoped review while the ownership preflight reports a blocker, naming it and the recovery
- [x] Unit: the archive candidate test asserts the refusal, that nothing is recorded, and that a pre-#1002 passing review is still not told to ship
- [x] Fixtures that are unowned by design stay verifying without a review
- [x] Integration: the ship-readiness test's first review is refused and names the recovery; the recovered change reviews and ships
