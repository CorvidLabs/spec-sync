---
kind: evidence
id: 2026-11-19-1038-red-control-for-the-archive-side-at-d14fcad3
aliases: []
claim: "Red-first control for the second parity finding on PR #1093, at d14fcad3 with only the new tests: deleted_package_files_beside_a_modified_archive_file_are_product fails (the tip reads archive_only), and the_own_package_exemption_agrees_with_the_ci_classifier_script fails on that same tip (the script says archive_only=false). Fixed in e54bc636; not a new bug."
result: failed
evidence_kind: product-test
revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
content: bf638a95bab9091bab5a1880c42b353e868c47abad1ea4614ebc0ac89c46416e
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:acedb02bb0aa7067bb452eb1bfcd4c80b3b45348345efdacb3fdcf8137f10e29
external: ext-b41337
criteria: [SHIP-8]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T06:20:35Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control for the second parity finding on PR #1093, at d14fcad3 with only the new tests: deleted_package_files_beside_a_modified_archive_file_are_product fails (the tip reads archive_only), and the_own_package_exemption_agrees_with_the_ci_classifier_script fails on that same tip (the script says archive_only=false). Fixed in e54bc636; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 5f5cdf8d41eb, observed 2026-11-19T06:20:35Z.
