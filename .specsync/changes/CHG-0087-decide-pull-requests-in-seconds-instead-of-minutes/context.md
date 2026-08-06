---
change: CHG-0087-decide-pull-requests-in-seconds-instead-of-minutes
artifact: context
---

# Context

Every failing pull request in the 6.0 cycle failed for one reason: a change
merged before it was finalized stayed active, and its verification commit stopped
being an ancestor of HEAD. Jobs ran in parallel with nothing gating anything, so
each pull request paid for the full matrix to rediscover a verdict that was
available immediately.

Ancestry is decidable with git alone, so the preflight needs no Rust toolchain
and no build. It is deliberately a fast FAIL and never a fast pass: every state
it cannot decide with certainty is left to `change audit --strict`, which stays
authoritative.

`trust` lives in its own workflow, so cross-workflow `needs:` does not exist and
the same audit runs inside it instead, immediately after the release build.
