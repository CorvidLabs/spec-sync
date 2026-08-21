---
change: the-release-lane-must-prove-a-candidate-before-running-its-code
artifact: plan
---

# Plan

1. `validate` — move the `merge-base --is-ancestor` proof ahead of `cargo metadata`, so nothing
   runs the candidate's own manifests before the candidate is known to be integrated.
2. Both `rust-cache` steps — `save-if: false`. Restoring a cache is harmless; saving one from a
   candidate tree on a default-branch-privileged run is the vector the rule describes.
3. `authorize-release` — keep its checkout, record why it is load-bearing, and note that any
   caching step added there later must carry `save-if: false` too.
