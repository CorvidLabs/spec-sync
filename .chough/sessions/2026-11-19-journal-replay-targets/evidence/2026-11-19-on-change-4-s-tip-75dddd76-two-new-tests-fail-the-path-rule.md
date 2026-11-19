---
kind: evidence
id: 2026-11-19-on-change-4-s-tip-75dddd76-two-new-tests-fail-the-path-rule
aliases: []
claim: "On change 4's tip 75dddd76: two new tests fail. The path rule checked the raw entry while the write turns backslashes into separators, so a journal entry .git\\config under specs_dir . and specs/.git\\config under specs were replayed; and specs_dir . admitted a Makefile entry."
result: failed
evidence_kind: product-test
method: cargo test --locked --bin specsync -- a_backslash_entry_cannot_reach_git_files specs_dir_dot_does_not_widen_replay (0 passed, 2 failed); rustc 1.89.0
revision: 75dddd7695acbbb2106feeb9889d315cd64297f8
content: 1328612c5b168699c443782e8132bd8e5964213052e8a2927f257d1255940881
environment: local-macos-arm64
criteria: [SETUP-8]
requirements: []
contracts: []
limitations: [Found from t4c's read of 94e4551d before Leif's review finished.]
observed: 2026-11-19T16:40:01Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On change 4's tip 75dddd76: two new tests fail. The path rule checked the raw entry while the write turns backslashes into separators, so a journal entry .git\config under specs_dir . and specs/.git\config under specs were replayed; and specs_dir . admitted a Makefile entry.**

Recorded as failed in local-macos-arm64 at revision 75dddd7695ac, observed 2026-11-19T16:40:01Z.

## Key failing lines

- called Result::unwrap_err() on an Ok value (the replay accepted the journal)
