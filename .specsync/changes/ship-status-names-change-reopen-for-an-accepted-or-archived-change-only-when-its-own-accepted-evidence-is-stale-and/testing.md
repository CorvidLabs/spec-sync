---
change: ship-status-names-change-reopen-for-an-accepted-or-archived-change-only-when-its-own-accepted-evidence-is-stale-and
artifact: testing
---

# Testing

| Behaviour | Evidence |
|---|---|
| Unrelated drift does not advertise reopen | `commands::change::tests::an_archived_change_is_not_told_to_reopen_for_unrelated_project_drift` (red before, green after) |
| Unreadable evidence names a restore | `commands::change::tests::an_archived_change_with_unreadable_verification_is_not_told_to_reopen` (red before, green after) |
| Stale accepted evidence still names reopen (REQ-cmd-change-033) | the existing per-state product-stage test, now given `Stale` evidence |

Locally: `cargo test --bin specsync commands::change::` and `--test integration ship`; hosted CI runs the full suite at the archived head.
