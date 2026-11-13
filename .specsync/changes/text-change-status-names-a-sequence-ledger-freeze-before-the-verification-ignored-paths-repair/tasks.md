---
change: text-change-status-names-a-sequence-ledger-freeze-before-the-verification-ignored-paths-repair
artifact: tasks
---

# Tasks

- [x] Red-first integration test: a frozen ledger plus an invalid ignored-path entry; text Next names the freeze, as JSON does; without the freeze, both name the policy repair
- [x] Move the sequence-freeze check before the verification_ignored_paths check in text_mode_next_action
- [x] Targeted tests, fmt and clippy on what changed
