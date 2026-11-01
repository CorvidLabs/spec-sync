---
change: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches
artifact: requirements
---

# Requirements

Preserve all receipt validation and resource bounds. Never authenticate a mismatched historical review. Recover only an exact already recorded package, retain its immutable preimage, and require fresh verification and scoped review before closing. Refuse modifications, absent or unrelated anchors, and other invalid digests without mutation.
