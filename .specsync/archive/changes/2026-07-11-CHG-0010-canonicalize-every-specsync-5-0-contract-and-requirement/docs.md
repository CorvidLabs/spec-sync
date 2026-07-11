---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: docs
---

# Docs

This change primarily updates canonical module documentation and companions. Configuration documentation inside the `config`, `cli`, and `cmd_rules` contracts must identify `.specsync/config.toml` as current, preserve legacy JSON/TOML compatibility truth, and avoid claiming TOML support for legacy-only `customRules`. The generator and repository-owned canonical config must share the exact version-neutral header `# spec-sync configuration`.

Public-facing README and site changes are out of scope unless implementation finds they repeat one of the corrected canonical falsehoods. Any such expansion requires an affected-path amendment and fresh definition approval.
