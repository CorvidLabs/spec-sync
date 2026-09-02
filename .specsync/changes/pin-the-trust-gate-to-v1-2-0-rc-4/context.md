---
change: pin-the-trust-gate-to-v1-2-0-rc-4
artifact: context
---

# Context

PR 750 pinned the hosted Trust action from CorvidLabs/trust@a239f786 (v1.1.1) to CorvidLabs/trust@e0272543 (v1.2.0-rc.4) in .github/workflows/trust.yml. The dogfood SpecSync 6.0.0 runner-local file:// mirror, specsync-version, and workflow shape are unchanged.

Lifecycle gate and Trust both require an active change covering that meaningful path. This package records the pin only; canonical spec text is not changing.
