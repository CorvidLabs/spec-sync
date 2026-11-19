---
change: change-correct-reads-its-summary-inside-a-change-read-scope
artifact: testing
---

# Testing

- `change_correct_costs_less_than_one_unscoped_summary` (src/change_tests/fast_archived_reads.rs):
  one accepted workflow-v1 change and two archived changes, persistent read cache deleted. Counts
  the Git processes of one whole `correct_interview_metadata` (live validation included) and of
  one `summarize_change` of the corrected change with no scope. Red on main: 696 against 641,
  the correction costing more than the unscoped summary it contains. Green: 279 against 641.
- Existing metadata-correction tests stay green; fmt and clippy clean.
