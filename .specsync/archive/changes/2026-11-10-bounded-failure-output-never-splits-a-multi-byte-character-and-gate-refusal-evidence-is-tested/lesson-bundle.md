# Lesson bundle: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Bounded failure output never splits a multi-byte character and gate-refusal evidence is tested
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change_tests.rs, src/change_tests/bounded_failure_output.rs, tests/integration/change.rs
- **Acceptance**: change check records findings over 4 KiB that contain multi-byte characters at the cut without panicking, keeping a tail that starts on a character and line boundary after a leading … line; the gate-refusal (effective contract) evidence entry's output carries the refusal texts and a test asserts it

## Evidence

- Verification commit: `d1c7cecb0fd0d6ba8898c3d15455b3fab70f9ef1`
- Base commit: `aa2f5108929c19a4e9eabd126985de609f709af1`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #1027 (Fixes #640), round 2. The corvid-agent review at `85056618` reproduced a panic:
`bounded_failure_output` sliced `joined[len-4096..]` by byte offset, so findings over
4 KiB whose cut lands inside `↔`, `—` or `…` (common in this repo's specs) panicked
instead of recording the failure #640 is about. The fix walks the cut forward to a
character boundary before looking for the next line break.

The review also found that REQ-change-151 AC3 (gate-refusal evidence carries the refusal
texts) had no test. The existing integration test that produces the
`(effective contract: auth)` entry now asserts its `output`. A separate fixture that only
drifted an export hit the spec↔code check, not the gate, so it was dropped.

The spec nits (invariant 52 below 51, the z=45 plane formatting) landed as a separate
commit before this change. Main's #1021 took z=44.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing specs run the full lifecycle with 0xLeif as approve and review actor. The added
acceptance criterion tightens this PR's own unmerged REQ-change-151 to the behavior its
first criterion already promised (a bounded record of findings, not a crash).

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/bounded_failure_output.rs` call `bounded_failure_output`
directly. The multi-byte test fails on the previous code with "byte index 3 is not a char
boundary". The integration test runs `change check` against a fixture whose declared
module drifts under strict enforcement.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-151 | unit `bounded_failure_output_respects_the_cap_at_a_char_boundary`, `bounded_failure_output_starts_on_a_line_boundary`, `bounded_failure_output_under_the_cap_is_verbatim`; integration `change_check_records_the_effective_contract_refusal_on_the_first_run` (gate-refusal `output`), `failed_change_check_records_findings_in_command_evidence` and `passing_change_check_records_no_output_key` from round 1 |

## Where these lessons go

- `specs/change/change.3md`
