---
change: tell-agents-when-it-is-safe-to-clear-context
artifact: testing
---

# Testing

## Discriminators

- `src/change_tests.rs` `handoff_*` — one test per `classify_handoff` branch: sequence freeze,
  archived, draft with questions, draft with stub artifacts, draft complete, stale approval,
  invalid correction ledger, accepted v2, accepted legacy stale/current, dirty scoped tree,
  verifying with stale verification, verifying awaiting review, verifying ready to finalize,
  approved clean.
- `src/change_tests.rs` `handoff_follows_the_lifecycle_and_ignores_uncommitted_lifecycle_evidence` — an uncommitted
  `review.json` alone leaves the handoff `safe`; an uncommitted edit under `affected_paths` makes
  it `conditional`.
- `src/change_tests.rs` `change_summary_carries_the_same_handoff_the_domain_computes` —
  `ChangeSummary.handoff` equals `handoff_summary` and serializes under `handoff`.
- `tests/integration/change.rs` `status_prints_a_handoff_line_and_json_carries_it` — text shows
  exactly one `Handoff:` line after `Next:`; `--json` carries `summary.handoff.readiness` on status
  and `handoff` on the approve transition.

## Control

- Existing `status` / `show` / `check` text assertions keep passing (the line is additive).
- No digest appears on the text line.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-093 | `src/change_tests.rs` `handoff_*` |
| REQ-cmd-change-005 | `tests/integration/change.rs` `status_prints_a_handoff_line_and_json_carries_it` |
| REQ-agents-check-audit-commands-001 | `src/agents.rs` `install_claude_creates_skill_and_command` asserting the handoff sentence |
