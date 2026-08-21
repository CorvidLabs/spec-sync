---
change: the-release-lane-must-prove-a-candidate-before-running-its-code
artifact: context
---

# Context

Closes #639. The release lane has never executed — #635 blocked it on a check nobody produced —
so the first real run of these steps will be the 6.0 RC. Fixing the ordering before the lane's
first execution is cheaper than discovering the exposure after it.

Three mistakes were made producing this change, all recorded because each is a real cost rather
than a story.

**A load-bearing checkout was nearly removed.** `authorize-release` opens with a checkout and a
`gh api` call that writes only to `$RUNNER_TEMP`, so the checkout looked unused. Eighty lines
further down — past the range that had been read — the same step runs `git show-ref`,
`git rev-parse HEAD`, and `validate-release-candidate.py --repository .`. Removing it would have
broken the release lane on its first execution, which is exactly when nobody would be looking
for a workflow bug.

**A mis-scoped first attempt could not be withdrawn.** It declared `src/change_tests.rs`
alongside the workflow under `--no-spec-change`; the ownership gate correctly refuses that,
because production source needs an owning module. But the gate fires at ship, three stages after
the interview, and by then the change was in `verifying` — past the scope freeze, with no way to
widen, withdraw or cancel. See #554 and #541.

**The obvious manual workaround corrupted the repository.** Deleting the stuck workspace by hand
removed a `state.json` that workflow-version history treats as an immutable creation anchor, and
every subsequent command refused with `workflow-version history deleted its immutable creation
anchor`. Recovery required resetting the branch and starting the change over. That escalates
#554 from "inconvenient" to "the workaround breaks the repo", which is worth knowing before
anyone else reaches for it.
