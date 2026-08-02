---
change: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
artifact: design
---

# Design

1. Keep the existing official GitHub Actions app, exact candidate SHA, successful conclusion, and
   external trusted-revision binding checks.
2. Enumerate bounded runs for the trusted lifecycle-policy workflow and select the unique successful
   `pull_request_target` run whose immutable fields match the repository, candidate, workflow path,
   trusted revision, and PR number.
3. Accept GitHub's canonical rewritten check URL without deriving authority from it.
4. Fail closed for zero matches, multiple matches, wrong app/event/path/repository/SHA/revision/PR,
   or unsuccessful runs.
