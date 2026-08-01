---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: docs
---

# Docs

- Add `docs/ci-confidence.md` as the operator-facing ownership matrix.
- Link it from `AGENTS.md` and `CONTRIBUTING.md`.
- State the everyday target as approximately 95% merge confidence, not an absolute guarantee.
- Distinguish the immediately delivered no-duplicate Trust change from the protected Tier B matrix
  follow-up so agents do not claim Windows/macOS have moved before the workflow PR lands.
- Explicitly forbid reintroducing full `lanes.verify` into hosted Trust.
