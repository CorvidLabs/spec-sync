---
change: approvals-forked-across-clones-are-refused-until-one-fresh-approval
artifact: research
---

# Research

- `ensure_committed_send_backs_kept` already walks committed ledger versions inside
  `ensure_definition_approval_valid`; the correction and retirement ledgers have their own fork
  checks, approvals had none.
- `change approve` accepts an approved, implementing or verifying change and appends a new
  definition approval without validating the old one, which is the recovery.
