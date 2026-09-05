---
change: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
artifact: testing
---

# Testing

- `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` — DISCRIMINATOR for the finalize defect: a legacy (v1) predecessor is accepted and committed, a v2 successor supersedes every `auth`-owned input of it, then approve → check → commit → check → review → finalize. Fails on ac796b8 with the field message; passes with the fix. Its tail proves the predecessor is successor-covered before and after the archive commit on BOTH `check_project` and `audit_project` (the audit's `terminal_evidence` rates the predecessor `SuccessorCovered`; on 507c14e4 the audit reported it stale with the "no successor" wording because its candidate universe was active-only), and that disturbing the successor's inputs afterwards yields the refused-successor diagnostic — identical on both surfaces — that steers to `change status <successor>` and away from `change reopen <predecessor>`.
- `stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication` — DISCRIMINATOR for the silent refusal: a covering v1 successor whose `verification.json` is flipped to `passed: false` is named with "its accepted evidence did not authenticate: accepted change has failed verification evidence". On ac796b8 the message claims no successor covers the input.
- `stale_accepted_change_error_names_covering_successor_with_stale_evidence` — CONTROL, re-anchored on the shared `accepted_change_with_covering_successor` fixture: the stale-successor case now carries the nested reason and keeps the verify-and-accept-or-reopen remediation for a v1/v1 pair.
- `stale_accepted_change_error_names_uncovered_input_and_reopen_remediation` and `stale_accepted_change_error_names_exact_only_input_and_audited_reopen` — CONTROLS, unchanged: the no-claiming-successor and exact-only messages are byte-identical.
- `fledge run lint` (clippy, `-D warnings`), `fledge lanes run pre-push`, `fledge lanes run verify`.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-020 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |
| REQ-change-024 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |
| REQ-change-audit-project-001 | `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` (audit assertions) |
| REQ-change-036 | `stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication`, `stale_accepted_change_error_names_covering_successor_with_stale_evidence`, `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change` |
