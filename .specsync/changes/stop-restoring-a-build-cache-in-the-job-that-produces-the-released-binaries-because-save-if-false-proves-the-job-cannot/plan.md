---
change: stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot
artifact: plan
---

# Plan

Recorded in `context.md` and `testing.md`: the reasoning error being corrected (`save-if: false`
proves the job cannot poison the cache, not that its output is trustworthy), why the `build` job
is the one that matters (its output is signed, published and installed by other people), why the
three sibling CodeQL alerts were dismissed rather than fixed, why `qualify` deliberately keeps its
cache and what open question that leaves, and why no discriminating test is possible for the
removal of a step.
