# Lesson bundle: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Show the steps between change new and approve on the landing page and name the change answer command in approve refusals (#883, D11)
- **Kind**: BugFix
- **Specs**: change
- **Paths**: site/src/content/docs/index.md, src/change/collect_requirement_ids.rs, src/change_tests/, src/change_tests.rs, specs/change/, tests/integration/documented_workflows.rs, .github/scripts/classify-ci-paths.sh, .github/scripts/test-classify-ci-paths.sh, CHANGELOG.md, .chough/
- **Acceptance**: The landing page Quick Start change workflow, run as written from its own init and add-spec auth lines with the by-hand steps it names, approves the definition it documents (the approved scope carries the documented acceptance criterion, spec auth and path src/auth.ts) and ends archived; the Archive gate refuses until finalize and passes wherever the page says to wait for CI (DOCS-3, DOCS-1).
- **Acceptance**: Approving right after change new refuses. The missing-acceptance-criterion refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`; the unfinished-interview refusal names every remaining question and the `specsync change answer <id> <question> <yes|no>` command for the first. Running each named command moves approval on until only the selected artifacts remain (DOCS-3.a, REQ-change-133).
- **Acceptance**: The approval guard is unchanged: every definition approve refused before is still refused; only the refusal text gains the command.
- **Acceptance**: site/src/content/docs/index.md classifies as a full-lane path and still a site path, so editing the landing page runs these tests in CI; other site pages keep site-only routing (DOCS-1.a).
- **Acceptance**: Both landing-page tests fail at the #881 archive tip 603a1356, because the page goes straight from change new to approve and the refusals name no command, and pass after the change; the README test keeps passing on the shared harness.

## Evidence

- Verification commit: `39ec1ee36bf79eacb0ec0f27d904acb7b7d69574`
- Base commit: `603a1356ae88763f5560c943361316456033ff49`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Audit finding D11 (#883, sealed audit of 2026-11-05 at 40dcf33a and 6.1.2 d08d2746). The landing page `site/src/content/docs/index.md` (Quick Start, lines 48-54) goes from `specsync change new "Add auth" ... --json` straight to `specsync change approve add-auth --actor "Ada"`. That approve refuses with `error: at least one acceptance criterion is required`. The guard is right; the snippet leaves out the interview and the selected artifacts. Reproduced locally with 6.2.0: after the criterion is answered, the next refusal is `error: the deterministic interview is incomplete`. Neither refusal names a command; only `change status` does.

Leif approved the intent on 2026-11-05 at 20:22 MST (orc tab, relayed by orc t18, "Approve all"). DOCS-3 and DOCS-3.a are this change. DOCS-3.a is a small product change to those two refusals. The landing page is also classified as a full-lane CI path, so editing it reruns its test.

This branch sits on #881's archive tip (603a1356, PR #898), because the shared harness is #881's `tests/integration/documented_workflows.rs`. origin/main is merged in once #898 lands.

REQ-change-133 comes from the orc's REQ-change registry (127 #861, 128 #890, 129 #889, 131-132 S6). Neither main nor any open PR branch had an id of 130 or above.

The artifact refusal (`artifact is incomplete: <path>`) and the delta refusal (`semantic delta modules must exactly match affected specs (missing: auth ...)`) already name what to edit. They are file edits, not commands, and are unchanged.

## From the change's design.md

# Design

## Refusals (`src/change/collect_requirement_ids.rs`, `validate_definition`)

Only the two refusal strings change; their conditions and order do not.

- No acceptance criterion: `at least one acceptance criterion is required; run \`specsync change answer <id> acceptance_criteria "<observable outcome>"\``
- Questions remaining (from `next_questions_for`, the same list that decides the refusal): `the deterministic interview is incomplete (still to answer: \`public_contract\`, \`architecture_risk\`); run \`specsync change answer <id> public_contract <yes|no>\``. The placeholder is the first question's choices joined with `|` when it offers two or more (`<yes|no>`), and `"<answer>"` otherwise (`constraints`, whose only suggestion is `none`, takes free text).

`validate_definition` also feeds the lifecycle audit's per-record errors; the same text appears there and stays accurate.

## Landing page (`site/src/content/docs/index.md`, Quick Start)

```bash
# Optional: verified change workflow (SDD). Off until:
specsync change adopt
specsync change new "Add auth" --spec auth --path src/auth.ts --json
# answer each question it returns, for example:
specsync change answer add-auth acceptance_criteria "login() returns true" --json
# then complete the selected artifacts and the auth delta (Workflow Guide)
specsync change approve add-auth --actor "Ada"
specsync change check add-auth --commit
# after the human implementation review:
specsync change review add-auth --reviewer "Ada"
specsync change finalize add-auth
# then push the archive commit, wait for CI, and merge
```

The closing line matches #881's archive-first order. The remaining interview answers are left to "answer each question it returns": `change answer --json` lists them, and approve now names the next one.

## CI route (`.github/scripts/classify-ci-paths.sh`)

A `site/src/content/docs/index.md` case before `site/*` sets both `full=true` and `site=true`, with a comment naming the product test that executes it. Other site paths keep site-only routing. `test-classify-ci-paths.sh` covers the landing page (full + site) and another docs page (site only).

## Tests (`tests/integration/documented_workflows.rs`)

The README test's loop becomes `Project::follow`, shared by all three tests. The prose rule for completing the definition now keys on "artifact", so the landing page's two-line wording is recognized; the README's line still matches.

- `landing_page_change_workflow_runs_from_its_quick_start` (DOCS-3, DOCS-1): it runs the block's own `specsync init` and `specsync add-spec auth` lines (writing `src/auth.ts` first, as the inline comment says) and commits that existing project. Then it follows every step after `# Optional: verified change workflow` as written, with the same Archive-gate rules as the README test (refuse while open, prove each named recovery, pass where it says to wait for CI). The approved scope in the archived `approvals.json` must carry the documented acceptance criterion, `["auth"]` and `["src/auth.ts"]`. The block's other lines (install, coverage, score, hooks, MCP) are a command list and are not run.
- `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` (DOCS-3.a): after the same prerequisites and the block's `change adopt` and `change new`, it runs the block's own approve line. Every refusal must name a `specsync change answer` command, or be the artifact refusal. The harness runs each named command, filling `"<observable outcome>"` with a sentence, `<yes|no>` with the recommended `no` and `"<answer>"` with `none`, and approves again. Any other placeholder fails the test. The first named command must be the acceptance-criteria answer, and at least two must be named.

## From the change's testing.md

# Testing

All three are in `tests/integration/documented_workflows.rs` and run against the built binary.

- `landing_page_change_workflow_runs_from_its_quick_start` fails at 9c28b332 (on #881's archive tip): the landing page's approve line fails when run as written, right after `change new`.
- `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` fails at 9c28b332: `error: at least one acceptance criterion is required` names no command.
- `readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci` keeps passing on the shared harness.
- `.github/scripts/test-classify-ci-paths.sh` fails at 9c28b332: `site/src/content/docs/index.md` classifies `full=false`.

After the change, all of these pass, along with `fledge lanes run verify` on the pinned rust 1.89.0 and `specsync check --strict`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-133 | `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them`, `landing_page_change_workflow_runs_from_its_quick_start` |

## Where these lessons go

- `specs/change/change.3md`
