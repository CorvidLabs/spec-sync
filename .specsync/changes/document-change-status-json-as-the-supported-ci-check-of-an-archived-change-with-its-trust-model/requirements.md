---
change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
artifact: requirements
---

# Requirements

ADDED REQ-cmd-change-038 (`deltas/cmd_change.md`): `change status <id> --json` is the documented CI
check of one archived change. It asserts `change.state` and `summary.terminal_evidence.validity`,
and the page states the trust model and `--timeout`. A test runs the page's own check.

Evidence: `archived_upstream::the_documented_ci_archive_check_passes_an_intact_archive_and_fails_a_tampered_one`
(`tests/integration/archived_upstream.rs`). It fails with the page absent (red run kept) and
passes with it.
