# Stage contract

Every stage uses the same shape so any agent can tell where work stands. The helper
embeds only the stage names, order and gates; this file holds the meaning. Use the
fields as a checklist for your own reasoning, not as a form to fill in front of the
human.

Each stage below states: purpose; inputs; outputs and their owners; useful roles
and tools; completion evidence; approval and preference policy; consequential
questions; what to checkpoint; and transitions.

## Transitions shared by every stage

| Intent | Command | Notes |
|---|---|---|
| Next stage | `chough stage advance` | Every move meets its destination's native prerequisite (build, test, accept, done) |
| Not applicable | `chough stage skip --reason "<why>"` | The reason is required and kept |
| Another pass | `chough stage iterate --note "<what changed>"` | |
| Go back | `chough stage back --to <stage> --reason "<why>"` | Lists what to refresh; history is kept |
| Start over | `chough stage reset [--to <stage>] --reason "<why>"` | History is kept; intent, design and plan are always reachable |
| Pause, resume | `chough stage pause --reason "<why>"`, `chough stage resume` | Resume only marks the stage active and checks nothing itself; then run `chough next --environment <env>` to re-check native state and freshness |
| Side stage | `chough stage enter <side> --reason "<why>"`, then `chough stage return` | Sides: bugs, validate, librarian; the caller is restored |

Going back to intent or plan after native approval means native reapproval if
intent, contract, acceptance criteria or scope change. Follow `specsync change
status`; Chough never changes native state. Dependent design approval, test
observations and evidence named in the refresh list must be redone by their owners.

## intent

- Purpose: capture what the human wants and derive an appropriately sized FRD.
- Inputs: conversation, `INTENT.md`, `hi export <scope>`, existing FRD and module requirements.
- Outputs: HI criteria (HI), FRD sections (`docs/FRD.md`), review findings (working notes).
- Roles and tools: product and user advocate, QA, design, accessibility, engineering, architecture; `hi`.
- Completion evidence: criteria captured with `hi`, FRD traced to them, findings synthesized.
- Approval: the human approves the intent stage as a whole; no per-criterion ceremony.
- Questions: only those that change the product or a consequential assumption.
- Checkpoint: criteria and requirement references, decisions, open questions.
- Transitions: advance to design; back is not available; reset restarts here.

## design

- Purpose: define visuals and interactions in one canonical local HTML and CSS prototype.
- Applicability: skip with a reason when the change has no relevant visuals or interactions.
- Inputs: approved intent, FRD, project conventions and design tokens.
- Outputs: canonical prototype and `COMPANION.md` (design owner), rendered review notes.
- Roles and tools: design, accessibility, user advocate; a browser-capable agent for rendered review.
- Completion evidence: rendered review of states and viewports, companion mappings to HI and FRD.
- Approval: the human approves the rendered prototype and companion together.
- Questions: design choices that change scope go back to intent.
- Checkpoint: prototype and companion paths in `references.design`, pending design approval in `awaiting_human`.
- Transitions: advance to plan; back to intent.

## plan

- Purpose: technical design, task plan, estimates and ownership in the native change.
- Inputs: intent, design, contracts, architecture, native lessons and principles.
- Outputs: native change package and deltas (SpecSync), estimates and ownership (plan artifact).
- Roles and tools: engineering, architecture, QA; `specsync change new/answer`.
- Completion evidence: native artifacts complete and status ready for approval.
- Approval: one human scope approval over the concrete definition, recorded natively.
- Questions: consequential open choices, batched into the one approval question.
- Checkpoint: `native_change`, the approval request in `awaiting_human`.
- Transitions: advance to build only after native approval; back to intent or design.

## build

- Purpose: implement the approved tasks, keeping contracts, tests and docs in sync.
- Inputs: approved native definition, owned files, case definitions.
- Outputs: code, tests, case definitions, documentation (project owners).
- Roles and tools: engineering; Fledge tasks and lanes; subagents with explicit file ownership.
- Completion evidence: focused checks pass locally, tasks updated, checkpoint saved.
- Approval: none for routine work inside scope; scope changes go back to plan.
- Questions: genuine blockers and scope-changing decisions only.
- Checkpoint: completed work, files, blockers, next action after each increment.
- Transitions: advance to test; back to plan, design or intent.

## test

- Purpose: focused verification of the affected criteria and behavior.
- Inputs: selected cases (`chough cases select`), project-required lane.
- Outputs: observations (`.chough/evidence/`), artifacts, bugs for failures.
- Roles and tools: QA; `chough cases run/record`; browser for design QA.
- Completion evidence: selected cases observed; failures captured; omissions explained.
- Approval: none; blocking bugs send work back to build.
- Questions: whether a gap is a product decision rather than a defect.
- Checkpoint: case and observation references, open bugs, test debt.
- Transitions: advance to accept (refused while blocking bugs are open); back to build.

## accept

- Purpose: present the implementation and evidence for the human's review, then close natively.
- Inputs: verification, product evidence, remaining bugs and limits.
- Outputs: native review and archive (SpecSync), lessons (native module sources).
- Roles and tools: the human reviewer; `specsync change check/review/ship/finalize`.
- Completion evidence: native state `archived`.
- Approval: the human implementation review, recorded natively only after it happens.
- Questions: the review itself.
- Checkpoint: the pending review in `awaiting_human`, PR and CI references.
- Transitions: advance to done only once archived; back to build or test.

## done

- Purpose: the change is archived. Merge follows the repository's process.
- Transitions: release only with authorization; reset to start new work.

## release

- Purpose: optional publication, installation or deployment with real project commands.
- Entry: a human request recorded as run consent, or a standing project setting.
- Outputs: distinct publication, installation, deployment and live-health observations.
- Transitions: advance to done.

## bugs

- Side stage for reproducing, classifying, deduplicating and recording a defect locally.
- Returns to its caller; blocking bugs send the main flow back to build.

## validate

- Side stage for optional repeated validation or monitoring.
- Entry needs a run consent or standing preference; findings are kept locally.

## librarian

- Side stage for adding to or organizing the optional project library and preparing briefs.
- Returns to its caller; the librarian never becomes a second current source.
