---
change: CHG-0088-catch-verification-evidence-a-squash-merge-discarded
artifact: context
---

# Context

The preflight was written to catch verification evidence orphaned by a squash
merge, and tested against a commit that was present but not an ancestor of HEAD.

A squash merge does not produce a non-ancestor commit. It produces an absent
one: nothing reachable points at the original commit any more, so a clone of the
target branch never fetches it. The script treated an absent commit as
shallow-clone ambiguity and skipped it, which meant the job passed while missing
the only case it existed for.

The job checks out with `fetch-depth: 0`, so every reachable commit is present.
An absent one was discarded by a squash, which is the defect rather than noise.

The original tests missed this because none of them modelled a commit that is
absent from the clone. Both failing cases had the commit present.
