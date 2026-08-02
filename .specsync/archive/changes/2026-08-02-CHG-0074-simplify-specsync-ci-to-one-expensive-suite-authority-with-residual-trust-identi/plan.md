---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: plan
---

# Plan

1. Split the hosted Trust lifecycle from the full local verification lane.
2. Point `.trust.toml` at the residual, no-test lifecycle lane.
3. Canonicalize the CI-versus-Trust ownership invariant in the GitHub spec.
4. Document confidence tiers, measured before/after expectations, and the protected-workflow
   follow-up required to move macOS, Windows, and expensive coverage off ordinary PRs.
5. Run the fast lane, strict spec validation, configuration assertions, and full local verification
   once before delivery; do not run staged-tip waiter loops.
