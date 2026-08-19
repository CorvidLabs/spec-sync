# Tasks

- [x] Enumerate every `deny_unknown_fields` and classify each as persisted evidence or regenerable cache
- [x] Remove it from the 17 persisted-evidence structs; keep it in the 4 cache structs
- [x] Identify which of the 17 are digest preimages, and state what tolerance does not buy
- [x] Replace all three unknown-workflow-version diagnostics
- [x] Add a forward-compat test, a cache-strictness control, and a diagnostic test
- [x] Confirm the CHG-0068 golden vector and every digest test are unchanged
