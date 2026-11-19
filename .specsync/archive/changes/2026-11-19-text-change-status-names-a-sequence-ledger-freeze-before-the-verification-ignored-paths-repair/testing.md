---
change: text-change-status-names-a-sequence-ledger-freeze-before-the-verification-ignored-paths-repair
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-034 | `a_sequence_ledger_freeze_outranks_the_verification_ignored_paths_repair_in_text_status` (tests/integration/archived_upstream.rs) |
| REQ-change-138 | the same test asserts JSON `next_action` names the freeze, then the policy repair once the freeze is cleared |

## How it runs

`change check` runs only the in-process spec-code sync. The integration test runs locally with
`cargo test --test integration a_sequence_ledger_freeze_outranks` (red before the fix, green
after), and in hosted CI's full suite at the archived head.
