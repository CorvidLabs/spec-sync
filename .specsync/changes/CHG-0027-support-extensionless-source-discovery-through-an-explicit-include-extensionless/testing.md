---
change: CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: testing
---

# Testing

Focused tests cover default, false, true, serialization, legacy JSON, ordinary extensions, and extensionless paths. End-to-end CLI fixtures create real canonical specs and source files, then assert exact non-zero file and LOC totals for extensionless-only and mixed projects under strict 100 percent enforcement.

- `REQ-config-003`: `config::tests::test_config_to_toml_roundtrips_include_extensionless`, `config::tests::test_toml_reads_explicit_false_include_extensionless`, and `config::tests::test_legacy_json_reads_include_extensionless` prove the canonical and legacy configuration contract.
- `REQ-validator-005`: `exports::configured_extension_tests` plus `check::extensionless_only_project_has_non_vacuous_strict_coverage` and `check::mixed_extensionless_project_has_non_vacuous_strict_coverage` prove consistent selection and measured strict coverage.

The final gate is `fledge lanes run verify`, followed by `specsync check --strict --require-coverage 100 --force`. Hosted CI must pass the packaged consumer, platform, compatibility, security, and regression matrices before merge.

Local results: the required Fledge verification lane passed formatting, Clippy, type checking, 1,567 unit tests, 195 integration tests, release build, and strict validation of 62 canonical specs at 100 percent file and LOC coverage with zero warnings. The broader repository lane additionally passed RustSec before the host's existing uninterruptible Bun I/O condition prevented its five parallel documentation and editor tasks from completing; those surfaces remain mandatory in hosted CI and are not recorded as locally successful.
