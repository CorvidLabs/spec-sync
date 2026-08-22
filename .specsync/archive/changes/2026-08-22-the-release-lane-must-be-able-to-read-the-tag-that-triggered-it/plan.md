---
change: the-release-lane-must-be-able-to-read-the-tag-that-triggered-it
artifact: plan
---

# Plan

Add `fetch-tags: true` to `resolve`'s checkout, with the measurement recorded beside it so the
setting is not later removed as redundant next to `fetch-depth: 0` — which is exactly what it
looks like.

Only `resolve` needs it: it is the one job that resolves and validates the RC tag itself. Every
later job checks out `candidate_sha`, a commit, and reads tags only through `git show-ref` for
the final tag, which the default fetch does provide.
