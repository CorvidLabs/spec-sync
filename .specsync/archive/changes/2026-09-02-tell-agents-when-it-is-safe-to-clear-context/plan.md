---
change: tell-agents-when-it-is-safe-to-clear-context
artifact: plan
---

# Plan

1. `src/change.rs` — add `HandoffReadiness`, `HandoffSummary`, `HandoffSignals`, pure
   `classify_handoff`, and `handoff_summary(root, record)` that gathers signals via the existing
   helpers (`sequence_ledger_freeze_next_action`, `next_questions`, `validate_artifacts`,
   `ensure_definition_approval_valid`, `recorded_verification_is_current`,
   `recorded_scoped_review_currency`, terminal-evidence summary) plus one new scoped
   `git status --porcelain -- <affected_paths>` probe that ignores `.specsync/`.
   Add `handoff: HandoffSummary` to `ChangeSummary`.
2. `src/commands/change.rs` — `print_handoff_line` after `Next:` in status, show, check (pass),
   approve, review, and both finalize sites (finalize and ship's finalize tail).
3. `src/agents.rs` — one sentence in `SKILL_BODY`; bump `AGENT_ARTIFACT_TEMPLATE_VERSION`;
   regenerate repo-owned skills.
4. Tests — unit test per `classify_handoff` branch, a repository test for the scoped dirty-tree
   probe, and an integration test that reads the text line and `summary.handoff` in JSON.
5. Docs — `site/src/content/docs/workflow.md` (new "Clearing context" section),
   `site/src/content/docs/cli.md` (status output), `CHANGELOG.md`.
