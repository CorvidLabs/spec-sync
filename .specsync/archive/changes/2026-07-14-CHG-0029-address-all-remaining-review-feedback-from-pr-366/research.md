---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: research
---

# Research

- `create_change` appends the sequence path before `next_questions` checks whether affected paths are empty.
- `record_covers_project_path` derives only conventional `specs/<module>/` scopes instead of using `canonical_module_paths`.
- `reject_direct_lifecycle_verification` treats any Cargo argument named `check`, `verify`, `change`, or `lifecycle` as a SpecSync invocation.
- `SddPolicy::default` protects selected `.specsync` files but omits the registry.
- `check_project_with_command_output` validates sequences before the disabled-policy return.
- `main.rs` guards only `change` and `lifecycle` before dispatch, leaving `check` to fail later in domain checking.
