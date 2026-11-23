---
change: keep-archived-coverage-recoverable-and-visible
artifact: testing
---

# Testing

Extend the existing real lifecycle fixtures. First archive a provisionally covered workflow-v2 change, commit it, then reopen, re-check, review and finalize. Check that the old receipt survives unchanged in Git history, the new archive is valid, and no old receipt leaks into a new generation with no temporary cover. Trigger a write failure after a journaled target and verify the original archived files and location can be restored. Verify an unexpected active receipt still blocks archive and remains untouched.

For the public CLI, exercise archived status/show and `audit --archives` in text and JSON. The cases are genuinely absent, valid with a withdrawn covering change, truncated JSON, wrong ID, unsupported schema, invalid entry, and oversized or unsafe input. Verify that the read commands make no project writes, that invalid input has a diagnostic, and that authenticated terminal status and the archive audit’s warning exit behavior remain intact.
