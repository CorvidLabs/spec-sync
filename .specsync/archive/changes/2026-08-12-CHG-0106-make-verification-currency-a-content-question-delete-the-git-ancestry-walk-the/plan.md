---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: plan
---

# Plan

1. Keep the content half of `verification_is_current_checked_with_project_digest`; delete the
   persistence-consistency call and the descendant walk that followed it.
2. Reduce `validate_verification_for_commit_binding` to its content half; drop the
   commit-identity and `merge-base --is-ancestor` arm and the `current_commit` parameter;
   update both call sites.
3. Delete the three orphaned functions whole.
4. Delete the unit tests that assert the removed behaviour rather than adapting them.
5. Update REQ-change-013 and REQ-change-016.
6. Record the squash fix, the deadlock fix, and the reduced change-then-revert detection in
   `CHANGELOG.md`.
