---
change: CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation
artifact: design
---

# Design

Keep the production change minimal: add a process-local atomic sequence to effective-contract scratch paths while retaining the process ID and timestamp for diagnostics. Prove uniqueness directly under parallel allocation, preserve cleanup after validation, and avoid a late 5.0 module split. Correct canonical companions and public docs to describe only shipped behavior. Preserve CHG-0002 as accepted evidence and record all corrections in this follow-up change.
