---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: research
---

# Research

tester2's runs on SpecSync 6.3.0 (`tester2/prs/g298/notes.md`): on six starling PR heads, 8 to 13
archived changes each read `corrupt-history` with another change's reason; synthetic S1, S2, S3 and
S6 show the same; S4, the control without the bad commit, reads `authenticated-history`. `check`,
`review`, `ship --dry-run` and `finalize` exit 0 in every case.

Read: `terminal_evidence_summary` and its three paths, `terminal_summary_from_cache`,
`archived_terminal_evidence_in_scope` (which already judges an archive against an empty successor
map, so the roster load is its only dependency on other changes), `validate_accepted_inputs_recursive`
(other records are read only when an input changed), `list_all_changes_loaded` and
`validate_retired_records`.
