# Lesson bundle — tell-agents-when-it-is-safe-to-clear-context

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Tell agents when it is safe to clear context
- **Kind**: Feature
- **Specs**: change, cmd_change, agents
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/agents.rs, tests/integration/change.rs, site/src/content/docs/workflow.md, site/src/content/docs/cli.md, CHANGELOG.md, specs/change/change.spec.md, specs/change/requirements.md, specs/change/testing.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/agents/agents.spec.md, specs/agents/requirements.md
- **Acceptance**: Every change status, show, check, approve, review, and finalize text result prints exactly one Handoff line that says safe, conditional, or not yet, gives one reason in plain language without digests, and when it is not safe names the concrete steps to take before clearing context.
- **Acceptance**: The JSON summary carries the same decision under summary.handoff with readiness, reason, resume, and before_clearing.
- **Acceptance**: Uncommitted edits under the change affected_paths make the handoff conditional; uncommitted lifecycle evidence under .specsync/ alone does not, because review then finalize is designed to run with that evidence uncommitted.
- **Acceptance**: A Draft change is never reported as safe; approval is the first clean boundary, and an approved change with a clean tree and current evidence is safe.
- **Acceptance**: A stale approval digest, a frozen sequence ledger, an invalid correction ledger, and stale legacy terminal evidence are all reported as not yet with the repair named.
- **Acceptance**: The handoff decision is a pure function of gathered lifecycle signals with a unit test per branch, and the installed agent skill tells agents to clear context only when the Handoff line says safe.

## Evidence

- Verification commit: `1a2ffb7ab967d8c6352cbcd6801b19cdf709d6d2`
- Base commit: `ddbc9343fa30be3a2def39f2e559ca9cf6984d2c`
- Verified by: `specsync check --spec agents --spec change --spec cmd_change`

## From the change's context.md

# Context

## What led here

Asked "is there any time I can safely clear context?", the honest answer for SpecSync 6 was
"only by reasoning about the lifecycle yourself". `change status` prints one next action but no
readiness verdict, so an agent has to know that approval pins the definition, that `check --commit`
pins verification, that review evidence is *supposed* to sit uncommitted until finalize, and that a
dirty tree under the change is the one thing a fresh session cannot recover. Agents that guessed
wrong lost work; agents that guessed right had no way to tell they were right.

## What a session picking this up needs to know

- The decision is a pure function of gathered lifecycle signals (`classify_handoff`) so every
  branch is unit-testable without a repository. `handoff_summary` gathers the signals from the
  same helpers `summarize_change_with_effective` already uses; it adds no new persistence.
- The dirty-tree signal is scoped to the change's `affected_paths` and ignores `.specsync/`.
  `review.json` is legitimately uncommitted between `change review` and `change finalize`
  (committing there stales the review), so whole-tree cleanliness would report `conditional`
  at exactly the moment the lifecycle wants the tree left alone.
- No digest is ever printed on the text line (CodeQL cleartext-logging rule already bit this
  repo once). Reasons are plain language.
- Approval is the first clean boundary: a Draft is never `safe`.
- `resume` is always `specsync change status <id>`; that command is the re-entry point and
  itself prints the Handoff line, so a fresh session can confirm the verdict it inherited.
- Agent guidance lives in `src/agents.rs` `SKILL_BODY` (regenerate the repo-owned skills with
  `specsync agents install` after editing; the template version advances so upgrades refresh).

## From the change's testing.md

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

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/agents/context.md`
