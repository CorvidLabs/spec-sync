---
description: Audit active SpecSync change workspaces and living specs (not archive history)
---

Audit active SpecSync change workspaces and living specs.

1. Run `specsync change audit`.
2. Report active-workspace and living-spec issues only. Archives are history — do not re-validate every archived CHG's terminal evidence.
3. Use this for "is the SDD workspace healthy?" not "did my feature tests pass?" (that is `change check` / `/specsync:check`).
