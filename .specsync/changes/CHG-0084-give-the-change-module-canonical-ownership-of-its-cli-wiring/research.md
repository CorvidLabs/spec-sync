---
change: CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring
artifact: research
---

# Research

`finalize` rejected CHG-0081 with `acceptance input `src/commands/change.rs` is
production source without deterministic canonical ownership`.

No spec claimed the file. Neighbouring modules claim their CLI wiring — the
`cmd_agents` spec claims `src/commands/agents.rs`, `changelog` claims
`src/commands/changelog.rs` — but the change module claimed only `src/change.rs`.

`check` does not enforce canonical ownership; `finalize` does. A change can
therefore verify green, pass every gate, sit accepted, and fail only at the
terminal step on a condition that held the whole time.
