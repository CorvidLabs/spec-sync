---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: tasks
---

# Tasks

1. Store the per-spec validation result with its hash entry.
2. Replay it for a spec skipped as unchanged; count it in `specs_checked`.
3. Re-validate and overwrite the stored result when a spec or its source changes.
4. Invert sandbox drill 038's #429 pin, which encodes the old behaviour.
