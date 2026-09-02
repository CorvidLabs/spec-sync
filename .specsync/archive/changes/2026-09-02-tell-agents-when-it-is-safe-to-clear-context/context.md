---
change: tell-agents-when-it-is-safe-to-clear-context
artifact: context
---

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
