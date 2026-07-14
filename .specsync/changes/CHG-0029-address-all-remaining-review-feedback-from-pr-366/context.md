---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: context
---

# Context

The final automated review of PR #366 identified six remaining lifecycle boundary defects after the PR had already merged:

1. The generated sequence-ledger path suppresses the interview question for real delivery scope.
2. Registry-resolved canonical spec and companion paths are not used consistently for path coverage and acceptance evidence.
3. `cargo run -- check` is rejected even when the selected binary is not SpecSync.
4. `.specsync/registry.toml` is not protected even though it controls canonical writes.
5. Sequence validation runs before an explicitly disabled SDD policy is honored.
6. Nested `check` reaches pre-dispatch work before the inherited verification guard rejects it.

These are all fail-closed correctness issues around the same boundary: lifecycle bookkeeping must not replace user intent, registry authority must be consistent, disabled policy must remain disabled, and recursion checks must reject only actual SpecSync lifecycle entry before handlers execute.
