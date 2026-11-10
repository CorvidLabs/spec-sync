---
change: failed-verification-evidence-carries-its-findings
artifact: testing
---

# Testing


Unit: failing scoped check records findings in `output`; passing record has no `output` key in
its JSON. Existing verification fixtures unchanged.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-151 | integration `failed_change_check_records_findings_in_command_evidence` (failed check records findings naming the missing file) and `passing_change_check_records_no_output_key` (passing record has no output key); old-format fixtures without the field parse unchanged |
