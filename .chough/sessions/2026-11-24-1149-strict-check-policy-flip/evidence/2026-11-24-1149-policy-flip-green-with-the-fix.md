---
kind: evidence
id: 2026-11-24-1149-policy-flip-green-with-the-fix
aliases: []
claim: "With the fix on the working tree over 2e2ddab1 (REQ-cmd-check-004, REQ-change-archived-requirement-loss-001): strict_check_keeps_the_comparison_when_the_branch_turns_sdd_off failed before the fix at check --strict and passes now; the disabled-at-base control passes both times. All 27 tests of the vanished_requirements integration module pass. Through the binary, the two policy-flip cases now exit 1 and name the loss in both check --strict and change audit --strict, uncommitted and committed, and the three controls are unchanged."
result: passed
evidence_kind: product-test
method: cargo test --test integration vanished_requirements, and the same real-CLI script as the red run, on the debug build of the working tree
revision: af9ee3727c15ef74371508dddabdaf168e540459
content: 49bef29e1d60ba763a9a8c85016936d3531321eb3553f2975ef5d54ff8764984
environment: macos-arm64-local
criteria: [CHANGE-17.d]
requirements: []
contracts: []
limitations: [macOS arm64 only; the full suite runs in the native Verify lane and on hosted CI]
observed: 2026-11-24T14:11:18Z
actor:
  - agent: claude-code
    terminal: t14v
---
**With the fix on the working tree over 2e2ddab1 (REQ-cmd-check-004, REQ-change-archived-requirement-loss-001): strict_check_keeps_the_comparison_when_the_branch_turns_sdd_off failed before the fix at check --strict and passes now; the disabled-at-base control passes both times. All 27 tests of the vanished_requirements integration module pass. Through the binary, the two policy-flip cases now exit 1 and name the loss in both check --strict and change audit --strict, uncommitted and committed, and the three controls are unchanged.**

Recorded as passed in macos-arm64-local at revision af9ee3727c15, observed 2026-11-24T14:11:18Z.
