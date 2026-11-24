---
change: keep-archived-coverage-recoverable-and-visible
artifact: design
---

# Design

Treat `coverage.json` as a generated receipt for one archive generation. A reopen from Archived may retire a bounded, structurally valid receipt belonging to that archive in the same durable file publication as the updated state, summary and reopen ledger. The journal must preserve the original bytes and recover them on write failure. Keep the existing confined path checks and archive directory rollback. A reopen from Accepted, an unexpected active copy, a corrupt receipt, or a receipt whose bytes change during the operation is never eligible for automatic cleanup. Restore or move aside unexpected data explicitly. Re-finalization runs the existing closing validation and writes only the new generation’s receipt, if any.

Use one bounded domain reader that distinguishes absence, valid coverage, and present invalid coverage. Check version, change ID, entry structure and canonical paths before looking up covering changes. Report the bad path and a safe restore instruction as an additive diagnostic. Keep the successful coverage rows and existing absent-file output shape. Archive authentication never consults this optional receipt.
