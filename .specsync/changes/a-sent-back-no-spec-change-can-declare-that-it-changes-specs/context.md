---
change: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
artifact: context
---

# Context

A change created with `--no-spec-change` that later turns out to change a spec had no way back:
`no_spec_change` could not be cleared, so the work needed a new change (#1070 item 2).

Leif's choice, 2026-11-13T03:52Z (coordinator chat, Claude Code hc t370, relayed by
claude-code-t370): for clearing `no_spec_change` he chose "Send back, then answer
(Recommended)": reuse the re-scope route, `change send-back`, then
`change answer <id> no_spec_change no` on the draft, then a fresh `change approve`.
