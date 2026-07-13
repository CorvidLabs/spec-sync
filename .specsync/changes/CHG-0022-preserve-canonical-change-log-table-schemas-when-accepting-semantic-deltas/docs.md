---
change: CHG-0022-preserve-canonical-change-log-table-schemas-when-accepting-semantic-deltas
artifact: docs
---

# Docs

No command syntax changes. Acceptance now preserves the canonical Change Log header already chosen by the
repository. Public lifecycle documentation remains accurate: each accepted semantic change increments the
canonical version and records its change ID without corrupting alternate table schemas.
