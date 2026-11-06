---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: requirements
---

# Requirements

REQ-change-123 (modified, change): a send-back records how many review attempts preceded it, and a scoped review recorded before the latest send-back is never current, even when the re-approved definition has the same digest. A send-back without that count falls back to a strict timestamp comparison.

No new requirement IDs.
