---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: testing
---

# Testing

- `REQ-change-039`: unit coverage proves a multi-entry batch appends contiguous sequences in one
  write, and that an invalid later entry leaves prior state bytes unchanged.
- `REQ-change-039`: unit coverage proves `--all-missing` selects only production-source affected
  paths that lack canonical owners and are owned by the named module.
- `REQ-cli-args-006` / `REQ-cmd-change-004`: CLI parse and integration coverage for repeated flags,
  manifest input, conflicting mode rejection, and JSON/text adapter output.
