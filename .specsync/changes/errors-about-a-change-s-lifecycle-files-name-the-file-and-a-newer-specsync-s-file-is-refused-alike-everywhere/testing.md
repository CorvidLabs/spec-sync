---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-164 | `lifecycle_file_errors::a_type_error_in_approvals_names_the_file` (red, then green) |
| REQ-change-165 | `lifecycle_file_errors::a_state_file_from_a_newer_specsync_is_refused_with_the_newer_specsync_message`, `a_verification_attempt_history_from_a_newer_specsync_is_refused_by_audit_and_status` (red, then green) |

Locally: `cargo test --test integration lifecycle_file_errors` and the change unit tests; hosted CI
runs the full suite at the archived head.
