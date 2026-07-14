---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: plan
---

# Plan

1. Separate generated sequence coverage from the user-supplied scope used by interview completeness.
2. Centralize registry-resolved canonical scopes for coverage and acceptance evidence, and protect the registry path.
3. Parse Cargo invocation boundaries so only an actual SpecSync target with a lifecycle subcommand is rejected.
4. Honor disabled policy before sequence validation.
5. Move the inherited recursion guard ahead of `check`, `change`, and `lifecycle` dispatch.
6. Add focused regressions for every review thread and update canonical `change` and `cli` contracts.
