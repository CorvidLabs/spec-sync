# Lesson bundle: failed-verification-evidence-carries-its-findings

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Failed verification evidence carries its findings
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, specs/change/, tests/, src/change_tests/
- **Acceptance**: a failed CommandEvidence in verification.json carries a bounded output field with the findings text; passing records serialize byte-identically to before; a flake vs regression is classifiable from the artifact alone

## Evidence

- Verification commit: `b55ae92cad468620728ed863d5f99f086fb33f22`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context






GitHub issue #640: a failed verification.json command recorded only command/success/exit_code,
so a flake and a regression were indistinguishable without a 10-minute re-run. The current
verification flow records the scoped spec↔code check instead of external commands, but the same
gap exists: `passed: false` with no findings text. The findings are computed but never persisted.

## From the change's testing.md

# Testing


Unit: failing scoped check records findings in `output`; passing record has no `output` key in
its JSON. Existing verification fixtures unchanged.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-151 | integration `failed_change_check_records_findings_in_command_evidence` (failed check records findings naming the missing file) and `passing_change_check_records_no_output_key` (passing record has no output key); old-format fixtures without the field parse unchanged |

## Where these lessons go

- `specs/change/change.3md`
