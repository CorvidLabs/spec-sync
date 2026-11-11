# Lesson bundle: a-failed-verification-records-the-findings-that-failed-it

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A failed verification records the findings that failed it
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/bounded_failure_output.rs, tests/integration/change.rs
- **Acceptance**: A failed verification command records a bounded output of the findings that failed it (errors, and warnings only under --strict), and a gate refusal records its refusal texts, so a recorded failure is classifiable without re-running it; passing records are byte-identical to before; the 4 KiB tail never splits a character and keeps the error that failed the check

## Evidence

- Verification commit: `af911335a6f1a239f8709a1c42f01fdef97122d4`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #640: a failed `CommandEvidence` carried only command, success and exit code, so a flake and a
regression could not be told apart without re-running the check. This change re-cuts PR #1027 (three
review rounds) as one fresh change on REQ-change-153, because #1027's archived rounds held
REQ-change-151, which coordinator t370 allocated to #1041 (2026-11-11). It adds requirement text, so
its definition approval is Leif's.

## From the change's design.md

# Design

`CommandEvidence.output: Option<String>` is serde-defaulted and skipped when `None`, so a passing
record keeps its bytes and an older record still parses. `bounded_failure_output(findings)` builds it
from the findings that failed the step: errors, plus warnings only when `--strict` made them
failures, and a gate refusal's texts. Over 4 KiB it keeps the tail: the cut moves forward to a
character boundary, then to a line boundary when one falls near the cut and text follows it, after a
leading `…` line. Empty finding text records no output.

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/bounded_failure_output.rs`; integration tests run a real `change check`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-153 | integration `failed_change_check_output_keeps_the_error_behind_many_warnings`, `failed_strict_change_check_output_records_the_warnings_that_failed_it` (fails without the `--strict` branch), `failed_change_check_records_findings_in_command_evidence`, `change_check_records_the_effective_contract_refusal_on_the_first_run`, `passing_change_check_records_no_output_key`; unit `bounded_failure_output_*`, `command_evidence_written_before_the_output_field_parses_without_output` (a record from an older binary parses with no output) |

## Where these lessons go

- `specs/change/change.3md`
