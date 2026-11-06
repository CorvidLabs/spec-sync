---
change: teach-archive-first-closeout-in-the-readme-and-run-its-workflow-example-as-a-test-881-d5
artifact: context
---

# Context

Audit finding D5 (#881, sealed audit of 2026-11-05 at 40dcf33a). README.md:168 tells the reader `# push after local gates; wait for required CI and complete human implementation review` before `change review` and `change finalize`. CI is archive-first: the `Archive` job runs `.github/scripts/archive-readiness.sh` before every other job and fails while any change is open under `.specsync/changes/`. A reader who follows the README waits on a CI run that can only be red.

AGENTS.md, `site/src/content/docs/workflow.md` and the generated agent skill already teach archive-first (`specs/agents/requirements.md`: "No generated text tells the agent to push a product tip and wait for remote CI before review"). README.md is the one place still wrong. Chough fixed its side in chough#122 (ec830cfb, PR #130); its wording is the reference:

> The hosted Archive gate refuses while any native change is open, so the closeout is archive-first: finish it locally before you wait on CI.
> A push before the review only shows the work early: expect the Archive gate red and do not wait on it.

Leif approved the intent on 2026-11-05 at 20:22 MST (orc tab, relayed by orc t18, "Approve all"): DOCS-1, DOCS-1.a, DOCS-2 and DOCS-2.a in `hi/docs.md` cover this change. DOCS-3 and DOCS-3.a are the landing-page change (#883), a separate change and PR.

Nothing else in the README changes, and no product code changes. The landing page (`site/src/content/docs/index.md`) is #883. `quickstart.md` does not tell anyone to wait for CI before review, so it stays as it is.
