---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: research
---

# Research

- `run_configured_command` (src/change/validate_loaded_change.rs) is the only code that sets
  `SPECSYNC_VERIFICATION_CONTEXT` on a child, and it is `#[allow(dead_code)]`.
- `verification_recursion_error` / `reject_recursive_lifecycle_dispatch` (src/main.rs) refuse a specsync
  process that inherits the marker. That process runs `check`, `change` or `lifecycle`, gets one
  diagnostic naming the parent, and exits 1.
- Tests: `indirect_recursive_lifecycle_check_fails_once_with_context` (check --strict) and
  `indirect_recursive_lifecycle_subcommands_fail_once_with_context` (change list, lifecycle enforce).
- `git grep -i "verification child"` on origin/main finds the cli module, the HLD row, and the
  change/cmd_comment text that #899 already handles.
