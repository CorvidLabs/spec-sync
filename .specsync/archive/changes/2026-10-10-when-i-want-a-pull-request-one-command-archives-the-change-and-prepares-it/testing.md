---
change: when-i-want-a-pull-request-one-command-archives-the-change-and-prepares-it
artifact: testing
---

# Testing

REQ-cmd-change-020
REQ-cli-args-019

- `change_pr_next_step_waits_for_green_checks_and_does_not_merge` in `src/commands/change.rs` pins the line the command prints after the pull request exists.
- Preparing this branch's existing pull request is the dogfood: `specsync change pr` prints that URL and does not open a second pull request.
