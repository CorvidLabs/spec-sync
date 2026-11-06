---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: design
---

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
