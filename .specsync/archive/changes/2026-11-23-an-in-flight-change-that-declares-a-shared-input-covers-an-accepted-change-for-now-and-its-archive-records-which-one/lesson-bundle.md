# Lesson bundle: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one
- **Kind**: Feature
- **Specs**: change, cmd_change
- **Paths**: src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/historical_reconciliation.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/retirement.rs, src/change/doctor.rs, src/change_tests.rs, src/change_tests/provisional_coverage.rs, src/commands/change.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, site/src/content/docs/, CHANGELOG.md
- **Acceptance**: An accepted change whose signed input a later approved, implementing or verifying change has edited, and whose module and path that change declares, reads `provisionally-covered` naming it; `change audit` and `change audit --strict` exit 0 with a warning naming both changes; with no such change it is stale and audit fails as before.
- **Acceptance**: `change archive`, or `change finalize` on workflow v2, accepts a provisionally covered change and writes `coverage.json` naming each covered input and its covering change; a provisionally covered change neither covers another change nor replaces one in a retirement, and `change reopen` still accepts it.
- **Acceptance**: An archived change that was covered for now reads `authenticated-history` whatever becomes of the covering change; when that change is sent back or closed, `change status`, `change show` and `change audit --archives` warn that the input has no accepted change behind it and name `change reopen`.
- **Acceptance**: Each of the two archive refusals over one stale delivery input names the other change and says it is blocked too, and the `change reopen` it names frees both.
- **Acceptance**: Released SpecSync 6.3.0 reads an archive that holds `coverage.json` as before: archived, `authenticated-history`, audit and doctor clean.

## Evidence

- Verification commit: `370d6671ea932fb589a06e0b66f6561614e53d50`
- Base commit: `a28e30420c12627a384a685cd5d808fc3585b70a`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

An accepted change goes stale the moment a later change edits one of its signed inputs, because only an accepted or archived successor covers the edit. While the later change is in flight, every accepted change that signed the same file fails `change audit --strict` at once; a team reopened six changes four times, about 20 minutes of re-verification each round (#657). When two accepted changes block each other's archive over one stale input, neither refusal says so (#688).

Leif's choices: for #657 on 2026-11-13 at about 04:04Z, "It may archive (Recommended)", so the archive records which change covered the shared file; for #688 on 2026-11-18, one sentence in each of the two archive refusals, built in the #657 PR. He approved this definition as written on 2026-11-19 at about 14:17Z ("Approve as written", relayed by hc t4c through ss-lead hc t58), with SHIP-13 and .a to .d as the agent's draft.

The unfinished branch `feat/657-provisional-coverage` (failing test `5063cf7d`, fix `195bca2d`, base bb7682a7) is the starting point; it carries no delta and nothing reads its `coverage.json`.

## From the change's design.md

# Design

- **Covered for now.** In the accepted-input walk, a changed input that no accepted or archived successor covers is covered for now by the first approved, implementing or verifying change created after the accepted one that declares the input's owning module and a path scope covering it. The walk returns `ProvisionallyCovered` and keeps each `(path, change)`.
- **Reported.** Terminal evidence reads `provisionally-covered` with a reason naming each path and its covering change. `change audit` adds a warning, never an error, so `--strict` exits 0; doctor treats it as covered, as the archive preflight does.
- **Not settled.** A provisionally covered change does not cover another change's input and is not a retirement replacement, because its own currency is only deferred. `change reopen` still accepts it.
- **May archive.** `change archive` and `change finalize` take the change's coverage from the closing check they already run, against the structural roster, and write `coverage.json` (schema version, change id, one `{path, change}` per input covered for now) into the package. The archive's own evidence never reads it.
- **Read back.** `change status` and `change show` on such an archive print each entry with the covering change's present state, and they and `change audit --archives` warn when that change is no longer approved, implementing, verifying, accepted or archived, naming `specsync change reopen <id> --actor <name> --reason <text>`. A warning, not an error.
- **#688.** The stale-input refusal adds one sentence naming each later change whose declared scope covers the input and saying it cannot be archived either; the preflight refusal adds one sentence saying the named change cannot be archived either until it is reopened or a covering change is accepted. Neither replaces the `change reopen` command.

Rejected: downgrading every stale accepted input to a warning, which would hide inputs nobody is working on.

## From the change's testing.md

# Testing

`src/change_tests/provisional_coverage.rs`, failing first:

- An accepted change, a later approved change declaring the same module and path, and a committed edit of the file: `provisionally-covered` naming the later change; audit has no error and one warning naming both; doctor lists no drift.
- The same edit with no declaring change: stale, and audit fails as before. A draft, or a change declaring the path without the owning module, does not cover.
- A provisionally covered change is not taken as a covering successor or a retirement replacement; `change reopen` accepts it.
- When the covering change is accepted, the earlier change reads `successor-covered`.
- The archive writes `coverage.json` naming the file and the covering change, through the literal `change archive` on a workflow-v1 fixture and through `change finalize` on workflow v2; another change's finalize is not refused because of it.
- After the archive, with the covering change sent back: the archive reads `authenticated-history`; status and show print the entry and the warning; `audit --archives` warns and exits 0. With the coverer accepted or archived, no warning. An archive with no `coverage.json` reads as before.
- REQ-change-168: both refusals name the other change and say it is blocked too, both still name `change reopen`, and the reopen frees both; with no later covering scope the first refusal reads as before.

Through the binary: `change audit --strict` exits 0 with the warning, the two refusals and the reopen that frees both. Released 6.3.0 against an archive holding `coverage.json`: status, audit, check and doctor clean. Full lane.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-167 | `src/change_tests/provisional_coverage.rs`: `an_in_flight_change_that_declares_the_input_covers_it_for_now`, `with_no_change_declaring_the_input_it_is_stale_as_before`, `only_an_in_flight_change_that_declares_the_module_and_path_covers_it`, `when_the_covering_change_is_accepted_the_earlier_one_is_successor_covered`, `a_provisionally_covered_change_can_still_be_reopened`, `a_provisionally_covered_change_is_no_retirement_replacement`, `a_provisionally_covered_change_covers_no_other_change`, `a_resumed_finalization_writes_coverage_json_like_change_archive`, `an_archive_warns_when_its_covering_change_is_sent_back`; released 6.3.0 and 6.4.0 against an archive holding `coverage.json` (the session's evidence) |
| REQ-change-168 | `src/change_tests/provisional_coverage.rs`: `the_two_archive_refusals_name_each_other_and_the_reopen_frees_both`, `with_no_later_covering_scope_the_stale_refusal_reads_as_before` |
| REQ-cmd-change-001 | `src/change_tests/provisional_coverage.rs`: `an_in_flight_change_that_declares_the_input_covers_it_for_now` and `an_archive_warns_when_its_covering_change_is_sent_back` read `provisionally-covered` and the recorded entries through the status projection |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
