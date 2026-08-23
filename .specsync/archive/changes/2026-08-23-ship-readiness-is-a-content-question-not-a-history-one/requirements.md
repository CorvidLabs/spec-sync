---
change: ship-readiness-is-a-content-question-not-a-history-one
artifact: requirements
---

# Requirements

No new requirement ID. `recorded_verification_is_current` is a new public export and is documented
in the `change` spec's Public API table, which is what `check --strict` gates on.

The behaviour it exposes is not new: `verification_is_current` has answered the content question
since the ancestry walk was removed from these paths, with the reasoning recorded inline at
`validate_verification_for_commit_binding` and `verification_is_current_checked_with_project_digest`.
This change makes that answer reachable from `ship-status`, which was still asking a different one.

No invariant is amended. The requirements describe currency as a content question already; the
defect was a caller that did not use it.
