---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: research
---

# Research

`validate_spec` currently checks file existence before its draft-sensitive section and export gates. `compute_coverage` already walks real source files, so missing paths do not need denominator filtering once the existence diagnostic is status-aware.

`ValidationResult` currently separates only errors and warnings; using warnings for planned mappings would make `--strict` fail and violate the requested behavior. A separate notice channel preserves deterministic output without weakening strict warning enforcement. The hand-written TOML reader and writer require explicit support for every new setting, while serde handles legacy camelCase JSON once the field is added to the known-key list.
