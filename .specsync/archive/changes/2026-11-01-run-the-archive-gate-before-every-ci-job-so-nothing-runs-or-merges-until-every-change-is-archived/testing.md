---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: testing
---

# Testing

- `python3 .github/scripts/test-required-ci-gate.py`: archive-first contract (Archive needs
  nothing, every job depends on it, both gates require it, an open change runs nothing and turns
  both gates red), the Archive job dogfoods `archive-gate-only`, and the Action runs its gate
  step first and skips every other step under `archive-gate-only`.
- `.github/scripts/test-classify-ci-paths.sh` and `test-archive-readiness.sh` still pass.
- CI `action-consumer`: the packaged Action passes `require-archived` on a clean consumer and
  blocks `archive-gate-only` once an open change is added.
- `cargo test --bin specsync -- github:: agents:: ship`: guidance, `Archive` trust relevance,
  installed agent text.
- `specsync check --strict --require-coverage 100 --force` and `change audit --strict`.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-github-023 | `.github/scripts/test-required-ci-gate.py` `ArchiveFirstTests` (Archive needs nothing, every job depends on it, an open change runs nothing and turns both gates red, the Archive job dogfoods `archive-gate-only`, the Action runs its gate first); `test-classify-ci-paths.sh` with the whole-PR classification; `github::tests` asserting `is_trust_relevant_check_name("Archive")` and not `"Archive old releases"`. |
| REQ-cmd-change-007 | `ship_guidance_teaches_archive_first_ci`: local guidance starts with archive-first and drops the product-tip/trust-reuse advice; stage ids stay `product_tip`, `review_tip`, `archive_tip`, `merge`; no stage title or action teaches waiting for trust or a review_only tip. `ship_status_and_finalize_agree_after_a_squash_that_preserves_content` stays green. |
| REQ-cmd-change-019 | `ArchiveFirstTests::test_an_open_change_runs_nothing_and_turns_both_gates_red` and `RequiredGateSimulationTests::test_any_selected_job_failing_or_cancelled_turns_the_required_gate_red` (Archive failing fails implementation ready and the required gate); `.github/scripts/test-archive-readiness.sh`; `status_says_which_open_change_to_archive_first`. |
| REQ-agents-009 | `tracked_agent_files_teach_archive_first_ci` over the tracked Claude, Codex, Cursor and Gemini skills and check commands, with `AGENT_ARTIFACT_TEMPLATE_VERSION >= 6`; `install_claude_creates_skill_and_command` and the other `agents::tests` stay green. |
