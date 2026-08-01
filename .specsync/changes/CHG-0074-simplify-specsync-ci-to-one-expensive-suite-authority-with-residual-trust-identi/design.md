---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: design
---

# Design

```text
GitHub CI (suite authority)       Trust (residual authority)
fmt / clippy / cargo test         release binary identity
strict spec coverage              contract on that binary
audit / coverage / consumers      Augur risk + Attest provenance

Local `lanes.verify` = complete suite retained for humans and agents
Hosted `lanes.trust-lifecycle` = lightweight lifecycle prerequisite only
```

The two hosted workflows provide complementary evidence. Trust does not treat the lightweight lane
as a substitute for CI, and CI does not replace binary identity, risk, or provenance.
