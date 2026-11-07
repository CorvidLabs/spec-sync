---
change: cover-the-two-paths-900-changed-outside-its-declared-scope
artifact: context
---

# Context

#900 (record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication, archived at 1a8d5048) changed two files that its affected paths did not list. On the archive tip `39d3d54d`, `change audit --strict` refused them in the CI Lifecycle gate and in Trust: "meaningful changed paths are not covered by an active change: src/change/collect_requirement_ids.rs, tests/integration/change.rs". The archived package cannot be amended, and AGENTS.md says a fix that changes the approved scope needs a new change.

- `src/change/collect_requirement_ids.rs`: `correction_prefix_digest` hashes each correction through `correction_digest_projection`, so embedded approvals' relays stay out of it. This is the Codex P1 fix in bbc46189, thread PRRT_kwDORqhp_860YaI9.
- `tests/integration/change.rs`: the exact top-level key-set test in `status_and_show_json_report_approved_delta_requirements` expects `recorded_claims` after approval and pins its value.

Leif reviewed both at 1b21d31f as part of #900. On 2026-11-06, relayed by orc t18, he said "Do the covering change": option A, with acceptance criteria CHANGE-6.d and CHANGE-13.b.

Ruled out: reverting either file (that would undo a P1 fix or fail a test) and editing the archived package.

The first cycle was approved at 4cdb2b36, verified at 31d657fd and reviewed by Leif. Then `change ship` refused: "acceptance input `src/change/collect_requirement_ids.rs` is production source without deterministic canonical ownership". With `--no-spec-change` and no affected spec, nothing owned that production file. `ship-status` had reported the change "ready to finalize". Leif said "Yes, add the spec" (2026-11-06, relayed by orc t18). The change was sent back, and `change` was added as its affected spec. Its files list owns `src/change/collect_requirement_ids.rs`, and `tests/integration/change.rs` is owned as a governed test. `--no-spec-change` stays, as in 27 archived changes.
