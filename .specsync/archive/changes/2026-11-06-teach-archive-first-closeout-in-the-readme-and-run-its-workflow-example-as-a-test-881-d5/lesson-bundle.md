# Lesson bundle: teach-archive-first-closeout-in-the-readme-and-run-its-workflow-example-as-a-test-881-d5

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Teach archive-first closeout in the README and run its workflow example as a test (#881, D5)
- **Kind**: Documentation
- **Paths**: README.md, tests/integration/documented_workflows.rs, tests/integration.rs, CHANGELOG.md, hi/docs.md, INTENT.md, .chough/
- **Acceptance**: Running the README Verified change workflow block as written, with the by-hand steps it names supplied, ends with the change archived, and every line in it that tells the reader to wait for CI comes at a point where the CI Archive gate passes (DOCS-1, DOCS-2).
- **Acceptance**: Until finalize the Archive gate refuses the open change; each distinct command its refusal names runs successfully in a copy of the project, and the named `change ship` then satisfies the gate.
- **Acceptance**: The new test fails on the README at ad50bdef, whose wait-for-CI line comes before the scoped review, and passes on the corrected README. It runs under cargo test, and README.md is a full-lane CI path, so CI fails when the example drifts (DOCS-1.a).
- **Acceptance**: The README closeout says what Chough teaches (chough ec830cfb): the hosted Archive gate refuses while a change is open, so finish locally before waiting on CI, and a push before the review only shows the work early. The no-commit-between-review-and-finalize rule and the human implementation review stay (DOCS-2.a).

## Evidence

- Verification commit: `5ef818a06edce327bc918abe67d2228fdbbba2c6`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Audit finding D5 (#881, sealed audit of 2026-11-05 at 40dcf33a). README.md:168 tells the reader `# push after local gates; wait for required CI and complete human implementation review` before `change review` and `change finalize`. CI is archive-first: the `Archive` job runs `.github/scripts/archive-readiness.sh` before every other job and fails while any change is open under `.specsync/changes/`. A reader who follows the README waits on a CI run that can only be red.

AGENTS.md, `site/src/content/docs/workflow.md` and the generated agent skill already teach archive-first (`specs/agents/requirements.md`: "No generated text tells the agent to push a product tip and wait for remote CI before review"). README.md is the one place still wrong. Chough fixed its side in chough#122 (ec830cfb, PR #130); its wording is the reference:

> The hosted Archive gate refuses while any native change is open, so the closeout is archive-first: finish it locally before you wait on CI.
> A push before the review only shows the work early: expect the Archive gate red and do not wait on it.

Leif approved the intent on 2026-11-05 at 20:22 MST (orc tab, relayed by orc t18, "Approve all"): DOCS-1, DOCS-1.a, DOCS-2 and DOCS-2.a in `hi/docs.md` cover this change. DOCS-3 and DOCS-3.a are the landing-page change (#883), a separate change and PR.

Nothing else in the README changes, and no product code changes. The landing page (`site/src/content/docs/index.md`) is #883. `quickstart.md` does not tell anyone to wait for CI before review, so it stays as it is.

## From the change's design.md

# Design

## README

The closeout block keeps every command it has today (`check --commit`, `review`, `finalize`) and the actors. Only the prose comments move:

```bash
specsync change check add-passkeys --commit
# run the local verify lane; CI does not test a tip whose change is still open
# after the human implementation review of the package, diff, delta and evidence:
specsync change review add-passkeys --reviewer "Ada"
specsync change finalize add-passkeys
# no commit between review and finalize; finalize commits the archive move
# then push the archive commit, wait for CI on that tip, and merge
```

The paragraph under the block gains Chough's two sentences: the Archive gate refuses while a change is open, so finish locally before waiting on CI, and a push before the review only shows the work early.

## Test: `tests/integration/documented_workflows.rs`

`readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci` reads the first `bash` block under `## Verified change workflow` from README.md at test time and splits it into commands (joining `\` continuations) and `#` prose steps.

- Every command runs exactly as written through `bash -e -o pipefail -c`, with the built `specsync` first on `PATH`, in a fresh git project. The project holds the README's `auth` module (`src/auth.rs`, one export, `REQ-auth-001`), created by `specsync init` and `specsync change adopt`. `SPECSYNC_*`, `GITHUB_EVENT_NAME` and `GITHUB_BASE_REF` are removed, as the `specsync()` helper does.
- A prose step that tells the reader to answer the interview and complete the artifacts is done the way a reader would do it. Each remaining question gets the answer the interview recommends. Every selected artifact gets real content: the plan has its required sections, tasks are all checked, and testing has a `## Requirement evidence` row. The module delta is written too. A prose step starting "implement" edits `src/auth.rs` without changing its export and commits it. Other prose needs no action; if the reader really had to do something there, the next command fails.
- A prose step that names CI or checks and says wait or observe runs the real Archive gate (`.github/scripts/archive-readiness.sh`), and the gate must pass there. That is the D5 assertion.
- After every step while the change workspace exists, the gate must refuse. Each distinct command the refusal names (`specsync change status <id>`, and `specsync change ship <id>` after the review) runs in a copy of the project. `status` must print a `Next:` line, and after `ship` the gate must pass in the copy. The test requires the `ship` recovery to have been exercised.
- The block must end with the change archived and the gate passing.

The actor and reviewer labels are fixture simulations, not human decisions.

The test is in `cargo test` (`tests/integration.rs`), so it runs in CI's `test` job and in `fledge lanes run verify`. README.md already classifies as a full-lane path (`classify-ci-paths.sh`, the `*` fallback), so a README-only edit reruns it.

The harness (block extraction, steps, `Project`) is written so the landing-page test in #883 can reuse it.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
