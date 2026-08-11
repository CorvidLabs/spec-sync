# SpecSync 6.0 WIP handoff — finish current changes, then taggable remaining work

You are working in the CorvidLabs/spec-sync Rust repo (workspace path as given).
Follow Agents.md / AGENTS.md strictly. Prefer fledge first. Enforcement is strict:
`specsync check --strict`, path coverage via active SDD changes, pre-push via
`fledge lanes run pre-push`. Never merge a PR while any SDD change is still active.

## Goal

1. Land the two active changes cleanly (CHG-0100 then CHG-0101, or reverse — but
   finalize ONE AT A TIME).
2. Report what remains before a clean `v6.0.0` tag (do not invent a tag without
   human approval).

## Current tree facts (verify with tools; do not trust this blindly)

- Branch: main, often ahead/behind origin — rebase/merge first if needed.
- Active changes:
  - CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
    (verifying; product tip done; needs independent review then finalize)
  - CHG-0101-add-audited-solo-maintainer-self-review-override
    (verifying; implementation largely done but may be UNCOMMITTED; no review yet)
- CHG-0101 intent: audited solo self-review:
  `specsync change review <id> --self-review --actor <scope-approver> --reason "…"`
  Must match scope approver; persist mode/reason; default remains independent
  `--reviewer`. Does NOT bypass verification, trust, CI, or finalize.
- PR #531 may exist for CHG-0100 finalize branch — reconcile with local main.
- Docs for 6.0 WIP (may be partially stale vs recent ship-status work):
  - docs/SESSION-SUMMARY-6-0.md
  - docs/GOAL-6-taggable.md
  - docs/GOAL-6-fixes.md
  - docs/6-0-findings.md
  - docs/6-0-tolerance-decisions.md (A/B/C DECIDED)

## Hard lifecycle rules (violations cost a full re-verify)

1. Review + ship are one step — no commit between review and finalize.
2. Finalize one change at a time — archiving stales sibling verification.
3. Do not batch reviews across changes.
4. Never merge with active changes still present.
5. Happy path: approve → implement → `change check --commit` → push product tip /
   wait trust when required → review → `change ship` / finalize → push archive tip
   → merge on GitHub.
6. For multi-active: use `change ship-status <id>` before acting.

## Immediate tasks (do in order)

### A. Orient

- `git status -sb`, `git log --oneline -10`, `git fetch` + compare origin/main
- `specsync change status` and `change ship-status` for both active IDs
- Read CHG-0101 change.md, state.json, tasks.md, testing.md, deltas/*
- Diff uncommitted work under src/change.rs, src/cli.rs, src/commands/change.rs
  and the three specs (change, cli_args, cmd_change)

### B. Finish CHG-0101 product work (if not already committed)

- Ensure CLI grammar, domain validation, persistence, status/ship rendering,
  and tests match REQ-change-046 / REQ-cmd-change-005 / REQ-cli-args-009
- Keep independent review default; self-review is narrow + audited
- Update Public API tables / requirements if exports or contracts changed
- Run focused tests: change::, commands::change::, cli::tests::
- `fledge lanes run pre-push` (or scripts/pre-push-gate.sh)
- `specsync change check CHG-0101-… --commit` so product tip evidence is on an
  ancestor of HEAD
- Do not invent second fake reviewer identities if self-review is ready —
  use the new flag after the feature binary is what you run

### C. Ship CHG-0100 and CHG-0101

- Prefer shipping the one whose product tip is already green first if 0101 still
  needs a re-check; otherwise finish 0101 evidence first then:
  - review (independent OR audited self-review once available)
  - ship/finalize WITHOUT intermediate commits
  - finalize siblings sequentially after re-anchoring if needed
- Reconcile PR #531 / any finalize branch with main; do not leave stranded active workspaces

### D. After both are archived — report taggable remaining work

Re-read docs/*6* and open issues. Confirm which of these are STILL true in code:

Still believed open / decision-needed:

1. Finding 4 — approve/check path vs commit window for approvals.json + change-sequence.json
2. Finding 7 — gate-side exemption for stub sections not authored by this change
   (NEVER wholesale StubSection filter; NEVER reword scaffold to dodge detector)
3. Finding 9 — depend satisfaction when Accepted is never observable post-finalize
4. GOAL-6 §1 design 2 — re-record verification at finalize (lock-free inner verify;
   deadlock if nested flock) — analysis only, not shipped
5. Warn at edit when approval/verification digest will invalidate
6. Sandbox candidate CI: unpin 5.2.0, build candidate SHA, SKIP ≠ PASS

Tolerance decisions A/B/C are DECIDED — implement if not already, do not re-litigate:

- A: scope draft-text failure to authored specs
- B: exempt inert/no-op changes from scoped review/finalize ceremony
- C: exempt metadata-only correct-owner from verification-command requirement

Open issues to triage for release-blocking vs later: #532, #530, #433, #428, #423, #439, #508, #498

## Standing rules for this codebase

- Drills / real binary paths judge lifecycle defects; green Rust suite alone is insufficient
  (single-process TempDir fixtures cannot see multi-clone / squash / lock issues)
- Invert sandbox drills in the same change as the fix
- Never `>/dev/null 2>&1` when probing failures
- `touch` sources before cargo build in worktrees if fingerprints miss script edits
- Elevate to human: digest/acceptance/archive semantics, approval digests, admin merges,
  any decision that weakens evidence
- Do not force-push main; do not weaken trust gates

## Deliverable back to me

1. What you shipped / committed / PRs updated
2. Exact `change status` + `ship-status` after your work
3. Remaining pre-tag checklist with evidence (fixed / open / needs human decision)
4. Any place you almost loosened a gate and what requirement stopped you

Start by reconciling git + both change workspaces; do not start new CHG-NNNN work unless
the two active ones are archived or you discover a true blocker that needs a follow-up change.
