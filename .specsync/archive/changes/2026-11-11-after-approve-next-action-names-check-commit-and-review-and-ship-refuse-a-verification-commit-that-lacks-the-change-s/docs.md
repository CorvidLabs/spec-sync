---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: docs
---

# Docs

No user-guide rewrite is in this change's affected paths. Once the
behavior is implemented, `change status` and `change show` themselves
carry the corrected next action, which is the guidance a user follows.
README and ADOPTING examples that already show `change check --commit`
do not need to change for this defect; any example that still shows a
bare `change check` as the step after approve should be updated in the
implementation pass if one is found.
