## ADDED

### REQUIREMENT REQ-cmd-change-archived-coverage-diagnostics-001

For an archived change, `change status`, `change show`, and `change audit --archives` SHALL report a present invalid `coverage.json` as an advisory warning, naming the affected archive, receipt path and safe restore action. Status and show SHALL expose the diagnostic in an optional JSON field and render the same warning in text. Archive audit SHALL include it in its existing warning list. A genuinely absent receipt SHALL produce no diagnostic. Successful coverage rows, the independently authenticated terminal state, and the exit conventions for archive warnings SHALL remain unchanged. These commands SHALL NOT repair or remove the receipt.

Acceptance Criteria
- The commands expose a warning for truncated, oversized, wrongly identified, unsupported, or unreadable receipts in both formats. Valid and genuinely absent controls preserve their current outputs.
- For an otherwise valid archive, reporting invalid auxiliary coverage does not change the terminal integrity classification or itself fail archive audit. Reporting it makes no project writes.
