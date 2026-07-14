---
change: CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: testing
---

# Testing

Focused tests cover default, false, true, serialization, legacy JSON, ordinary extensions, extensionless paths, and the wizard regular-file boundary. End-to-end CLI fixtures create real canonical specs and source files, then assert exact non-zero file and LOC totals for extensionless-only and mixed projects under strict 100 percent enforcement.

- `REQ-config-003`: `config::tests::test_config_to_toml_roundtrips_include_extensionless`, `config::tests::test_toml_reads_explicit_false_include_extensionless`, and `config::tests::test_legacy_json_reads_include_extensionless` prove the canonical and legacy configuration contract.
- `REQ-validator-005`: `exports::configured_extension_tests`, `commands::wizard::tests::wizard_source_candidates_exclude_matching_directories`, `check::extensionless_only_project_has_non_vacuous_strict_coverage`, and `check::mixed_extensionless_project_has_non_vacuous_strict_coverage` prove consistent selection and measured strict coverage.

The final local gate is `fledge lanes run verify`, followed by `specsync check --strict --require-coverage 100 --force`. Hosted CI must pass the packaged consumer, platform, compatibility, security, and regression matrices before merge. Results are recorded only after those commands complete.
