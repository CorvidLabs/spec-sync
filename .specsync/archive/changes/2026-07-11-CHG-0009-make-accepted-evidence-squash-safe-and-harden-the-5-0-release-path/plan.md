---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: plan
---

# Plan

1. Specify and implement remote-integrated squash evidence semantics.
2. Add exact topology and tamper regressions.
3. Archive the six merged accepted records using the fixed binary against the clean merge tree.
4. Harden exact and floating release tag behavior.
5. Verify locally, publish one focused PR, and require green main before tagging.
