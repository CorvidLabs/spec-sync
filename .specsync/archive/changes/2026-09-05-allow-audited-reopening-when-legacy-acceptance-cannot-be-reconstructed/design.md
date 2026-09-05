---
change: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
artifact: design
---

# Design

Add a distinct ReopenCauseV1 variant for unreconstructible manifest-less workflow-v1 acceptance. Evaluate reconstruction after existing evidence authentication, and do not let exact/successor input checks veto that recovery cause. Preserve all prior evidence and require fresh verification and closing. Modern records do not use this legacy path.
