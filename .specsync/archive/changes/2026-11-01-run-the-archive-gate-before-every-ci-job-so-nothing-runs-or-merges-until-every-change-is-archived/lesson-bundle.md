# Lesson bundle — run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run the archive gate before every CI job so nothing runs or merges until every change is archived
- **Kind**: Operations
- **Specs**: github, cmd_change, agents
- **Paths**: .claude/commands/specsync/check.md, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/commands/specsync-check.md, .cursor/skills/spec-sync/SKILL.md, .gemini/commands/specsync/check.toml, .gemini/skills/spec-sync/SKILL.md, .github/scripts/select-ci-lane.sh, .github/scripts/test-classify-ci-paths.sh, .github/scripts/test-required-ci-gate.py, .github/workflows/ci.yml, .github/workflows/trust.yml, AGENTS.md, CHANGELOG.md, action.yml, docs/ADOPTING.md, docs/HLD.md, docs/ci-confidence.md, site/src/content/docs/cli.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/workflow.md, site/src/content/examples/ci-gate.mdx, src/agents.rs, src/commands/change.rs, src/github.rs
- **Acceptance**: A tip with any open change fails the Archive job first in CI and Trust, every other job is skipped, and the required CI gate fails; a tip with every change archived runs the full lane selected from the whole pull request; the Action's require-archived and archive-gate-only inputs run the same gate before downloading SpecSync; ship-status and agent guidance describe verify, review and ship locally, then push the archive tip.

## Evidence

- Verification commit: `6a0028581fa6f816700be461dc93297cfc0eed40`
- Base commit: `24e29e100930db10e2f9db78bb7bcbf5ab6507e4`
- Verified by: `specsync check --spec agents --spec cmd_change --spec github`

## From the change's context.md

# Context

Every SDD change must be archived before its pull request merges. CI already had an `Archive`
job and the required gate needed it, but it ran beside everything else on purpose: the old ship
flow pushed a product tip with the change still open, waited for tests and `SpecSync
implementation ready`, then pushed review/archive tips that reused the parent's green run. The
owner (0xLeif) asked for archival to block CI before anything runs, then to roll the same gate
out to every CorvidLabs repository that uses spec-sync (Raven, Chough and the rest).

Ruled out: keeping product-tip CI while also gating (it still runs CI before archival), and
gating only other repositories (spec-sync would teach a flow it does not follow itself).

A mid-flight session needs to know: the archived tip now carries all CI evidence, so local
`change check --commit` and `fledge lanes run verify` are the product verification before ship.
Published `@v6.1.1` predates the new Action inputs and ignores them, so other repositories
must pin a ref that contains this change.

## From the change's design.md

# Design

The gate is `.github/scripts/archive-readiness.sh`, unchanged: it needs only Python and the
checkout, so it runs before any download or build. The Action runs it from
`${{ github.action_path }}` as its first step; `archive-gate-only` skips the download, check and
comment steps. CI's `archive` job calls the Action with `uses: ./`, so CI and every consumer run
the same gate.

Job graph: `archive` → `classify`/`preflight` → everything else; `implementation-gate` and
`ci-gate` run with `always()` and fail when `archive` did not succeed. A skipped required check
counts as passing on GitHub, so consumers must require an aggregate `always()` job, never a job
the gate can skip.

Classification uses the whole-PR diff only. With no open change reaching classify, tip-only
narrowing could only skip the product lane on the archived tip, which is now the only tip CI
tests. `review_required` can no longer be true past the gate, so corvid-pet selects on `full`.

Ship guidance keeps every JSON field and stage id; only text changes, and check-runs are read
on HEAD first because the archived tip is where CI decides the merge.

## From the change's testing.md

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

## Where these lessons go

- `specs/github/github.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/agents/agents.3md`
